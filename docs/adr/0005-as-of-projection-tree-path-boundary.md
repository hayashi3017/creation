# ADR 0005: As-Of Projection と Current TreePath の役割分離

- Status: `Accepted`
- Last updated: `2026-04-25`

## Context

家系図の relationship は時系列を持つ。

現在の主要な前提は次の通り。

- `person` は `birth_date` / `death_date` を持つ
- `relationship` は `start_date` / `end_date` / `deleted_at` を持つ
- `tree_path` は `ancestor_id` / `descendant_id` / `depth` だけを持つ
- `tree_path` には `valid_from` / `valid_to` のような時点情報がない
- 現在の `tree_path` は current-state の lineage closure として維持される

RFC 0013 では、保存する relationship を canonical relationship に寄せる方針を定義した。

初期の tree-edge kind は次の 2 つに限定する。

- `parent`
- `adoptive_parent`

初期の lineage closure に含めないものは次の通り。

- `step_parent`
- `spouse`
- `partner`
- `cohabitant`

RFC 0014 では、`GET /api/family-trees/{diagram_id}?as_of=1995-01-01` のような as-of family-tree projection を提案している。

この ADR では、その as-of projection が current-state の `tree_path` を historical source of truth として使うべきかどうかを決定する。

## Problem

as-of mode では、指定日付時点で有効な人物と関係だけを使って家系図を投影したい。

しかし current-state の `tree_path` は時点情報を持たないため、過去時点の lineage closure を正しく表現できない。

例:

```text
A -> B: 1990-01-01 から 2000-12-31 まで有効
B -> C: 1995-01-01 から 2010-12-31 まで有効
```

この場合、`A -> C` の ancestor / descendant 関係は 1995-01-01 から 2000-12-31 までだけ有効になる。

current-state の `tree_path` はこの期間を表現できない。

さらに複数経路がある場合、同じ ancestor / descendant ペアに複数の有効期間が発生しうる。

例:

```text
A -> B: 1990-01-01 から 2000-12-31 まで有効
B -> C: 1995-01-01 から 2010-12-31 まで有効
A -> D: 1980-01-01 から 2010-12-31 まで有効
D -> C: 1980-01-01 から 1992-12-31 まで有効
```

この場合、`A -> C` は少なくとも次の 2 区間で有効になる。

- 1980-01-01 から 1992-12-31
- 1995-01-01 から 2000-12-31

単純な current-state closure や単一期間の closure row では、この意味を安全に扱えない。

## Requirements

as-of projection では次を満たす必要がある。

- current-state ではなく指定日付時点の graph を返す
- canonical relationship rows を `as_of` で解釈する
- `birth_date > as_of` の person は除外する
- `death_date < as_of` の person は家系図上は visible のままにできる
- relationship validity は inclusive range として扱う
- as-of lineage graph に対して cycle detection を行う
- as-of read は `tree_path` を mutate しない
- `deleted_at` は通常 read から除外する administrative deletion として扱う

relationship の as-of 有効判定は次とする。

```text
(start_date IS NULL OR start_date <= as_of)
AND
(end_date IS NULL OR as_of <= end_date)
AND
deleted_at IS NULL
```

`birth_date IS NULL` の person は visible として扱う。

`death_date < as_of` の person は除外せず、deceased であることを UI 側で表現できるようにする。

## Decision

current-state の `tree_path` は as-of historical source of truth として使わない。

as-of mode では、canonical relationship rows を `as_of` でフィルタし、その request 内で lineage closure を構築する。

採用する方針:

- `tree_path` は current-state closure optimization として維持する
- current mode では既存の `tree_path` を利用してよい
- as-of mode では current `tree_path` を closure source として利用しない
- as-of mode の source of truth は date-filtered canonical relationship rows とする
- as-of lineage closure は request-local に計算する
- initial tree-edge kinds は `parent` と `adoptive_parent` に限定する
- `step_parent` は initial lineage closure に含めない
- `spouse` / `partner` / `cohabitant` は lineage closure に含めない
- temporal `tree_path` は初期導入しない
- 性能問題が profiling で確認された場合だけ temporal closure cache を検討する

`divorced_spouse` kind は採用しない。

離婚や死別などの終了理由は、次のような lifecycle metadata として表現する。

- `kind = spouse`
- `start_date`
- `end_date`
- `end_reason = divorce | death | separation | unknown`

as-of projection の active relation 判定には `end_date` を使う。

`end_reason` は validity 判定ではなく、説明や historical annotation に使う。

## Considered Options

### Option A: Current TreePath を As-Of にも使う

利点:

- 既存の `tree_path` を使い回せる
- 実装が一見単純に見える
- current mode と as-of mode の closure source が同じになる

欠点:

- `tree_path` に時点情報がない
- historical truth を誤る可能性が高い
- relationship の期間変更を正しく反映できない
- 複数経路・複数期間を表現できない
- 正しさの検証が難しい

判断:

- 採用しない

### Option B: Request-Local As-Of Projection を使う

利点:

- canonical facts を query-time に素直に解釈できる
- current `tree_path` の意味を壊さない
- write-side への影響が小さい
- temporal closure maintenance を初期導入しなくてよい
- correctness を read-side の入力と出力でテストしやすい
- RFC 0012 の `KinshipDerivationService` と相性がよい

欠点:

- as-of read は current-state read より重くなる可能性がある
- request ごとに lineage closure を計算する必要がある
- read-side に graph derivation logic が必要になる

判断:

- 採用する

### Option C: Temporal TreePath を導入する

利点:

- historical closure を事前計算できる可能性がある
- as-of query を高速化できる可能性がある
- 大規模 graph では有効な cache になりうる

欠点:

- 実装と保守が重い
- retroactive update の影響範囲が広い
- path validity interval の分割・結合が必要になる
- 同じ ancestor / descendant ペアに複数区間が発生しうる
- depth 再計算が複雑になる
- migration / repair / debugging の難度が高い
- 初期導入には過剰

判断:

- 初期導入では採用しない
- profiling で必要性が確認された場合だけ再検討する

## Consequences

良い影響:

- as-of projection の正しさを current `tree_path` の状態に依存させずに済む
- `tree_path` の役割が current-state optimization に限定される
- historical projection を canonical facts の query-time interpretation として扱える
- temporal closure maintenance の複雑さを初期実装から外せる
- `KinshipDerivationService` に read-side graph interpretation を閉じ込めやすい

悪い影響:

- as-of read は request ごとの計算量が増える
- current mode と as-of mode で closure source が異なる
- as-of graph 専用の cycle detection が必要になる
- large diagram では性能問題が出る可能性がある

設計上の注意:

- as-of read は `tree_path` を更新してはいけない
- historical projection は別途永続化された historical graph ではない
- `deleted_at` の意味を historical audit のために変えない
- audit mode が必要な場合は archive/event log を別途設計する

## Rollout

### Phase 1

`GET /api/family-trees/{diagram_id}?as_of=YYYY-MM-DD` の read path を追加する。

この段階では次だけを行う。

- person/entity を diagram から読み込む
- `birth_date > as_of` の person を除外する
- relationship を `as_of` でフィルタする
- endpoint が visible でない relationship を除外する
- direct explicit edge の projection を返す
- roots / adjacency を as-of graph から再計算する
- as-of mode では current `tree_path` を使わない分岐を入れる

### Phase 2

as-of lineage graph から次を導出する。

- `ancestor`
- `descendant`
- `sibling`

この段階で、request-local lineage closure builder を導入する。

### Phase 3

上位 kinship を導出する。

- `uncle_aunt`
- `nephew_niece`
- `cousin`

必要に応じて center person 起点の kinship endpoint を追加する。

### Phase 4

profiling で必要性が確認された場合だけ temporal closure cache を検討する。

候補:

```sql
CREATE TABLE temporal_tree_path (
  diagram_id BIGINT NOT NULL,
  ancestor_id BIGINT NOT NULL,
  descendant_id BIGINT NOT NULL,
  depth INT NOT NULL,
  valid_from DATE,
  valid_to DATE,
  PRIMARY KEY (diagram_id, ancestor_id, descendant_id, valid_from)
);
```

この table は初期導入しない。

## Future Extensions

将来の拡張余地:

- `end_reason` の enum 化
- audit mode 用の archive/event log
- temporal closure cache
- year-only / month-only の曖昧な historical date
- ancestry view と household view の分離
- former spouse を active relation ではなく historical annotation として返す endpoint

## Open Questions

- `deleted_at` 済み row を audit mode で参照する必要があるか
- date precision は `DATE` だけでよいか、year-only / month-only を扱うか
- `step_parent` を将来 opt-in projection で lineage に含めるか
- former spouse を as-of active relation から除外した後、どの endpoint で historical annotation として返すか
- temporal closure cache が必要になった場合、diagram 全体で持つか、date bucket ごとに持つか

## Summary

as-of projection is a query-time interpretation of canonical facts, not a separately persisted historical graph.

We will keep `tree_path` as a current-state closure optimization and will not use it as the historical source of truth for as-of family-tree reads.

For as-of mode, we will filter canonical relationship rows by date and derive lineage closure request-locally.
