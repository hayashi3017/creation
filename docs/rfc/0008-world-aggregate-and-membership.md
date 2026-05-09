# RFC 0008: World Aggregate、Entity 所属、Diagram 所属

- 状態: `下書き`
- 最終更新: `2026-04-26`

## 背景

現在の model では `diagram` が `entity`、`person`、`relationship`、`tree_path` の top-level container である。

この構造は単一 diagram の編集には十分だが、次の要件には不足する。

- 複数の family-tree diagram を 1 つの管理単位にまとめたい。
- `world` 内に複数の `diagram` を定義したい。
- `person` / `entity` は diagram ではなく world に所属させたい。
- Genealogy Overview API では、world 内の diagram を統合した家系図を返したい。
- diagram 単体は編集単位として残したい。
- world 単位で後から visibility、publication、collaboration を扱えるようにしたい。

今後は `world` を上位 aggregate の名前として採用する。`genealogy` は API / read model の語彙として使い、world 自体の名前にはしない。

Relationship と `tree_path` の最終 scope は RFC 0025 で world-level に更新する。この RFC の初期記述では relationship を diagram-scoped fact として扱っている箇所があるが、RFC 0025 採用後は `relationship.world_id` を canonical source とし、diagram は entity 表示範囲として扱う。

## 目標

- `world` を複数 diagram と entity を束ねる top-level aggregate として追加する。
- `entity` は world に所属する canonical node として扱う。
- `person` は world-scoped `entity` の specialization として維持する。
- `diagram` は world に所属する表示・編集単位として維持する。
- world 内の diagram と entity を read-time に統合できるよう、deep-copy merge を避ける。
- 未リリース前提で、最終形の schema と実装順序を定義する。

## 非目標

- Genealogy Overview response contract の詳細をこの RFC で決めること。
- World CRUD endpoint の詳細をこの RFC で決めること。
- entity merge / split UI の実装。
- ACL、collaborator、publication scope の最終設計。
- Genealogy Overview API の詳細な response contract。

## 提案

`world` を `diagram` と `entity` の上位 aggregate として追加する。

概念:

- `world`: 複数 diagram と人物 entity を束ねる単位。
- `entity`: world に所属する canonical node。`kind = person` の entity が人物を表す。
- `person`: world-scoped `entity` の specialization。
- `diagram`: world 内 entity を選択する表示・編集単位。relationship は所有しない。
- `genealogy_overview`: world 内の diagram を統合して返す read model。

推奨 schema:

```sql
CREATE TABLE world (
    world_id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE diagram (
    diagram_id BIGSERIAL PRIMARY KEY,
    world_id BIGINT NOT NULL REFERENCES world(world_id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    kind diagram_kind NOT NULL,
    description TEXT,
    genealogy_overview_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE entity (
    entity_id BIGSERIAL PRIMARY KEY,
    world_id BIGINT NOT NULL REFERENCES world(world_id) ON DELETE CASCADE,
    kind entity_kind NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);
```

`entity.diagram_id` は追加しない。entity は world に直接所属し、diagram で利用する entity は中間テーブルで管理する。

## 所属ルール

- すべての diagram は 1 つの world に所属する。
- すべての entity は 1 つの world に所属する。
- `entity.diagram_id` は持たない。
- diagram を別 world に移動する操作は提供しない。
- world を soft-delete すると、その world に属する diagram / entity / person / relationship / diagram_entity / tree_path も同じ transaction で soft-delete または削除する。
- diagram の soft-delete は world 自体を削除しない。

Diagram の `kind` は維持する。overview の対象は、初期実装では `kind = family_tree` かつ `genealogy_overview_enabled = true` の diagram のみに限定する。familytree という外部 API 命名は RFC 0017 で genealogy に統一する。

`genealogy_overview_enabled` は diagram 単位で Genealogy Overview への出力を制御する設定である。false の diagram は通常の diagram CRUD / edit では利用できるが、Genealogy Overview の read projection からは除外する。

## Diagram と Entity の対応

Entity が world に所属するようになると、diagram は entity を所有しない。

diagram で利用する entity は多対多の関係になるため、中間テーブルで明示的に管理する。

推奨名は `diagram_entity` とする。

理由:

- 既存 table 名が単数形なので `diagram_entity` が一貫する。
- どの 2 table の関連かが名前から直接分かる。
- `j_` などの接頭語は project 内で意味が定義されておらず、検索性と可読性が落ちる。
- join table であることは composite primary key と foreign key で十分表現できる。

別候補:

- `diagram_entity_membership`: 中間テーブルであることは分かりやすいが長い。
- `diagram_entity_link`: link table であることは分かるが、既存命名より抽象的。
- `diagram_person`: 現在は person だけなら分かりやすいが、将来 entity kind が増えると狭すぎる。

推奨 schema:

```sql
CREATE TABLE diagram_entity (
    diagram_id BIGINT NOT NULL REFERENCES diagram(diagram_id) ON DELETE CASCADE,
    entity_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    PRIMARY KEY (diagram_id, entity_id)
);
```

Relationship scope は RFC 0025 で world-level に更新する。relationship は diagram 内 fact ではなく、world 内 entity 間の explicit fact として保存する。

```sql
CREATE TABLE relationship (
    relationship_id BIGSERIAL PRIMARY KEY,
    world_id BIGINT NOT NULL REFERENCES world(world_id) ON DELETE CASCADE,
    source_entity_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    target_entity_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    kind relationship_kind NOT NULL,
    start_date DATE,
    end_date DATE,
    end_reason VARCHAR(32),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);
```

制約:

- `diagram.world_id` と `entity.world_id` は一致している必要がある。
- `diagram_entity` の diagram と entity は同じ world に属する必要がある。
- relationship の `world_id` と source / target entity の `world_id` は一致している必要がある。
- diagram projection では、diagram に登録された entity 同士の active relationship を自動表示対象にする。
- 同じ world 内の entity は、複数 diagram に `diagram_entity` として登録できる。
- 同じ人物を複数 diagram で表現したい場合、新しい entity を作らず同じ `entity_id` を複数 diagram に登録する。
- relationship を持たない孤立 entity でも、`diagram_entity` に登録されていれば diagram-specific projection に node として出せる。

PostgreSQL で world 一致制約を直接表しにくい場合は、service / repository 層で検証する。

## Identity 統合ルール

同一人物の canonical key は `entity_id` である。別途 `world_person` table は追加しない。

例:

```text
world_id = 1
entity_id = 10

diagram A: diagram_entity(diagram_id = 100, entity_id = 10)
diagram B: diagram_entity(diagram_id = 200, entity_id = 10)
```

Genealogy Overview では `entity_id = 10` を 1 node として返す。

もし同じ実人物を誤って別 entity として作成した場合は、後続の merge operation で片方へ統合する。初期実装では自動判定しない。

## Relationship endpoint

Relationship は引き続き entity 間の explicit fact として保存する。

```text
relationship.source_entity_id -> entity.entity_id
relationship.target_entity_id -> entity.entity_id
```

追加ルール:

- relationship の `world_id` と、source / target entity の `world_id` は一致する必要がある。
- diagram read では source / target entity がどちらも対象 diagram に `diagram_entity` として登録されている場合に edge として表示する。
- overview では relationship endpoint を変換せず、そのまま `entity_id` node 間の edge として扱う。

この設計では `world_person_id` への正規化処理が不要になる。

## Read-time 統合方針

World 内の diagram 統合は deep-copy ではなく read-time projection とする。

理由:

- diagram entity membership の更新を overview に自然に反映できる。
- copy 後の drift を避けられる。
- entity の diagram membership provenance を保持しやすい。
- 統合ルールを read model として段階的に改善できる。

欠点:

- read path が複雑になる。
- diagram 間で共有される entity の source provenance を扱う必要がある。
- relationship は world-level に寄せるため diagram 間の重複 edge merge は不要になるが、world-level fact 自体の conflict policy は別途必要になる。

## API との関係

未リリース前提のため、legacy compatibility は考慮しない。

- 新 API は RFC 0017 に従い genealogy 命名を使う。
- diagram CRUD は world_id を必須にする。
- entity/person CRUD は world-scoped に変更する。
- world を省略した diagram / entity / person create は提供しない。

## Migration 方針

未リリース前提で DB を作り直すため、既存 data の backfill や互換 migration は不要である。

初期 schema に次を直接含める。

1. `world`
2. `diagram.world_id NOT NULL`
3. `entity.world_id NOT NULL`
4. `diagram_entity`
5. `entity.diagram_id` の削除
6. `diagram.genealogy_overview_enabled BOOLEAN NOT NULL DEFAULT true`

変更時は [docs/local-development.md](../local-development.md) の `Database Reset` に従い、DB を削除して再作成する。

## World Delete 方針

World delete は関連 data へ即時に伝播する soft-delete とする。

同じ transaction で更新する対象:

- `world.deleted_at`
- `diagram.deleted_at`
- `entity.deleted_at`
- `person.deleted_at`
- `relationship.deleted_at`
- `diagram_entity.deleted_at`

`tree_path` は soft-delete column を持たないため、対象 world の entity に紐づく rows を削除する。`tree_path` は current-state closure cache であり、履歴 fact ではない。

理由:

- DB state と API visibility を一致させる。
- deleted world 配下の stale relationship / tree_path が read path に残ることを避ける。
- restore を初期実装の対象外にできる。

World restore が必要になった場合は、別 RFC で lifecycle policy を定義する。

## 実装順序

1. `world` table と model を追加する。
2. `diagram.world_id` と `entity.world_id` を初期 schema に追加する。
3. `entity.diagram_id` を schema から削除する。
4. `diagram_entity` を追加する。
5. diagram create に world_id を要求する。
6. entity/person create に world_id を要求する。
7. diagram に entity を追加する API / repository を追加する。
8. relationship write で relationship world と endpoint entity の world 一致を検証する。
9. world delete で関連 data を同じ transaction で soft-delete する。
10. world 内 diagram 一覧と entity 一覧 read を追加する。
11. overview 用に world 内の family_tree diagram、diagram_entity、world-level relationship を load する repository を追加する。
12. RFC 0015 の overview projection を実装する。

## Test Plan

最小 coverage:

- world 内に diagram を作成できる。
- diagram ごとに `genealogy_overview_enabled` を設定できる。
- world 内に person entity を作成できる。
- diagram に同じ world 内の entity を追加できる。
- world が異なる entity は diagram に追加できない。
- relationship endpoint entity は relationship と同じ world に属する必要がある。
- diagram に登録されていない entity を endpoint に持つ relationship は、その diagram projection には表示されない。
- world delete が関連 diagram / entity / person / relationship / diagram_entity を同じ transaction で soft-delete する。
- world delete が関連 tree_path rows を削除する。
- overview では同じ entity が複数 diagram に存在しても 1 node になる。

## 未解決事項

- world-level visibility や publication scope をどの table に置くか。
- entity merge / split API をどの RFC で扱うか。
- world restore を将来提供するか。
