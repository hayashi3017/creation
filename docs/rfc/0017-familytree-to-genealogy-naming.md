# RFC 0017: FamilyTree から Genealogy への命名変更

- 状態: `下書き`
- 最終更新: `2026-04-26`

## 背景

現在の repository には `family_tree` / `FamilyTree` / `family-trees` という語彙が複数箇所にある。

World と overview の導入後は、単一 diagram の家系図だけでなく、world 内の複数 diagram を統合した genealogy graph を扱う。今後の API と code の中心概念は `family tree` より `genealogy` の方が適切である。

また、overview API は world-scoped graph を返すため、`world_id` を path parameter として表現し、filter 条件だけを query string で渡す。

## 目標

- repository 全体で familytree / family_tree / family-tree の public-facing 命名を genealogy に寄せる。
- Overview API を `/api/genealogy/world/{world_id}` に統一する。
- `world_id` は path parameter で渡す。
- 既存実装から段階的に rename できる対象範囲を定義する。
- DB enum value と API path の互換性をどう扱うか明確にする。

## 非目標

- World / entity 所属 model の詳細実装。
- Overview projection の詳細実装。
- UI 文言の最終決定。
- 既存 API の versioning policy 全体。

## 命名方針

推奨語彙:

- Domain/API concept: `Genealogy`
- Overview read model: `GenealogyOverview`
- Endpoint: `/api/genealogy/world/{world_id}`
- World-scoped graph: `genealogy graph`

避ける語彙:

- `FamilyTreeOverview`
- `/api/worlds/{world_id}/family-tree-overview`
- `/api/family-trees` の新規追加

既存の単一 diagram API は移行中だけ残してよい。

## API 方針

Overview endpoint:

```http
GET /api/genealogy/world/{world_id}
```

Query:

```http
GET /api/genealogy/world/1?diagram_ids=10,11&center_entity_id=100&ancestor_depth=3&descendant_depth=2&as_of=1995-01-01
```

Response の data object は RFC 0015 に従う。

`GET /api/genealogy/world/{world_id}` を採用する。world resource の read API として path で scope を明示し、cache、共有URL、debug を扱いやすくするため。

## Rename 対象

段階的に次を rename する。

- route path: `/api/family-trees/{diagram_id}` から genealogy 系 endpoint へ移行
- driver DTO: `FamilyTree*` から `Genealogy*`
- usecase: `FamilyTreeUsecase` から `GenealogyUsecase` または `GenealogyOverviewUsecase`
- service output type: `FamilyTree` / `FamilyTreeNode` / `FamilyTreeEdge` から `GenealogyGraph` / `GenealogyNode` / `GenealogyEdge`
- docs: family-tree / familytree 表記を genealogy に統一

ただし DB enum `diagram_kind = 'family_tree'` はすぐに rename しなくてよい。DB enum value の rename は migration と既存 fixtures への影響が大きいため、別実装 step で扱う。

## DB enum の扱い

短期:

- `diagram.kind = 'family_tree'` は維持する。
- API と code の public naming は genealogy に寄せる。

長期候補:

```sql
CREATE TYPE diagram_kind AS ENUM (
    'genealogy',
    'correlation'
);
```

未リリース前提で DB を作り直すタイミングなら、`family_tree` を `genealogy` に置き換えてよい。ただし既存 code と fixtures の rename が同時に必要になる。

## 移行方針

1. 新規 API は `/api/genealogy/...` のみで追加する。
2. Overview は `/api/genealogy/world/{world_id}` とし、`world_id` は path parameter で受け取る。
3. 既存 `/api/family-trees/{diagram_id}` は互換用として残すか、未リリース前提で削除する。
4. Rust type / module / tests を `Genealogy*` に rename する。
5. docs と RFC の新規記述では genealogy を使う。
6. DB enum rename は実装時にまとめて判断する。

## Test Plan

最小 coverage:

- `/api/genealogy/world/{world_id}` が world-scoped overview を返す。
- filter query なし request は全体 overview を返す。
- query string の `world_id` は要求しない。
- 新規 DTO / response type は `Genealogy*` 命名を使う。
- 既存 family-tree endpoint を残す場合、deprecated 扱いであることを docs に明記する。

## 未解決事項

- DB enum `diagram_kind` の `family_tree` を初回実装で `genealogy` に変更するか。
- 既存 `/api/family-trees/{diagram_id}` を削除するか deprecated として残すか。
- `GenealogyUsecase` と `GenealogyOverviewUsecase` のどちらを主要 usecase 名にするか。
