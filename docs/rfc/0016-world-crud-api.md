# RFC 0016: World CRUD API

- 状態: `下書き`
- 最終更新: `2026-04-26`

## 背景

RFC 0008 で `world` を diagram と entity の上位 aggregate として導入する。`world` を実際に利用するには CRUD API が必要である。

World CRUD は単純な resource CRUD に見えるが、diagram、entity/person、Genealogy Overview の lifecycle に影響するため、先に API と deletion semantics を定義する。

## 目標

- `world` の作成、一覧、取得、更新、削除 API を追加する。
- diagram create が world に紐づく前提を整える。
- soft-delete semantics を定義する。
- Genealogy Overview API の input になる `world_id` を管理できるようにする。

## 非目標

- ACL / collaborator の実装。
- publication / share URL の実装。
- person CRUD の詳細。
- diagram move API の実装。
- hard delete API の提供。

## 提案 API

既存 API 形に合わせ、World resource endpoint は write 系を action-style route として追加する。

```http
GET /api/worlds
POST /api/worlds/create
GET /api/worlds/{world_id}
PATCH /api/worlds/update/{world_id}
DELETE /api/worlds/delete/{world_id}
```

Create request:

```json
{
  "name": "Hayashi family",
  "description": "Family research workspace"
}
```

Response:

```json
{
  "status": "success",
  "data": {
    "world_id": 1,
    "name": "Hayashi family",
    "description": "Family research workspace",
    "created_at": "2026-04-26T00:00:00Z",
    "updated_at": "2026-04-26T00:00:00Z"
  }
}
```

## Validation

World の validation は既存 RFC 0003 の name / description rule に従う。

- `name` は trim 後に空なら reject する。
- `name` は DB column size に合わせた最大長を持つ。
- `description` は trim し、空文字列なら `NULL` に正規化する。
- soft-delete 済み world は通常 read で `404` とする。

## Diagram Create との関係

Diagram は world に所属するため、diagram create request は `world_id` を要求する。

推奨 endpoint:

```http
POST /api/worlds/{world_id}/diagrams
```

既存の `POST /api/diagrams` を残す場合でも、request body に `world_id` を要求する。

```json
{
  "world_id": 1,
  "name": "Main tree",
  "kind": "family_tree",
  "description": null
}
```

未リリースのため、world なし diagram create の backward compatibility は不要とする。

## Delete Semantics

World delete は soft-delete とする。

方針:

- `world.deleted_at` を設定する。
- 所属 diagram / entity / person / relationship / diagram_entity は同じ transaction で即時に soft-delete する。
- 関連 `tree_path` rows は削除する。
- overview read は deleted world を `404` とする。

理由:

- DB state と API visibility を一致させる。
- deleted world 配下の stale relationship / tree_path が read path に残ることを避ける。
- lifecycle ownership を world に集約しやすい。

注意点:

- repository query は world の active 条件を必ず join で確認する。
- hard delete が必要な場合は運用 tool または別 admin API として設計する。

## 一覧 API

`GET /api/worlds` は active world のみを返す。

初期 sort:

```text
ORDER BY updated_at DESC, world_id DESC
```

将来 pagination を追加する。

```http
GET /api/worlds?limit=50&cursor=...
```

初期実装では件数が小さい前提なら pagination は省略してよい。

## Repository / Usecase 境界

推奨追加:

- `WorldService`: validation と world CRUD semantics を担当する。
- `WorldUsecase`: driver からの request を受け、service / repository を orchestrate する。
- `WorldRepository`: `world` table の persistence を担当する。

Diagram create は `WorldRepository.exists_active_world(world_id)` を使って world の存在を検証する。

`GenealogyOverviewUsecase` は world CRUD service ではなく read repository を使って world と所属 diagram / entity を load する。

## Test Plan

最小 coverage:

- world create が name / description を正規化する。
- world list が deleted world を返さない。
- world get が active world を返す。
- world get が deleted / missing world を `404` にする。
- world update が name / description を更新する。
- world delete が world と関連 diagram / entity / person / relationship / diagram_entity の `deleted_at` を設定する。
- world delete が関連 tree_path rows を削除する。
- deleted world に対する overview read は `404`。
- diagram create は missing / deleted world_id を reject する。
- diagram list / overview は world_id で scope される。

## 未解決事項

- world restore API を初期実装に含めるか。
- world list に pagination を初期実装で入れるか。
- diagram create endpoint を `/api/worlds/{world_id}/diagrams` に一本化するか。
- ACL 導入時に world ownership をどの table で表現するか。
