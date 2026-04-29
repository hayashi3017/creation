# RFC 0018: Resource Authorization Boundary

- 状態: `下書き`
- 最終更新: `2026-04-29`

## 背景

現在の API は認証 middleware によりログイン済み user を確認しているが、`world_id`、`diagram_id`、`entity_id`、`relationship_id` などの path parameter / query parameter が、その user からアクセス可能な resource かどうかを一貫して検証していない。

そのため、ログイン済み user が他 user の resource ID を推測または取得できた場合、他 user の world / diagram / genealogy projection を取得・更新・削除できる可能性がある。

これは path に ID を含めること自体の問題ではない。ID は resource locator であり、認可条件ではない。各 read / write path は、ID で resource を探すだけでなく、actor user がその resource に対して必要な permission を持つことを検証する必要がある。

## 問題

代表的な問題:

- `GET /api/worlds/{world_id}` が `world_id` のみで world を取得すると、他 user の world を読める。
- `GET /api/genealogy/diagram/{diagram_id}` が `diagram_id` のみで diagram を取得すると、他 user の diagram projection を読める。
- `GET /api/genealogy/world/{world_id}` が `world_id` のみで overview を構築すると、他 user の world-scoped graph を読める。
- `diagram_ids` query に他 user の diagram ID を混ぜた場合、repository query が actor scope で絞られていないと情報が混入する。
- update / delete 系で resource ownership を確認しない場合、他 user の data を変更できる。

この種の問題は IDOR / BOLA として扱う。

## 目標

- すべての resource ID access に actor user scope を適用する。
- world / diagram / entity / relationship の read / write permission 境界を定義する。
- 外部 authorization SaaS を利用する場合でも、application 側に残すべき境界を明確にする。
- user 側の認可機能、collaborator、share、publication scope と両立できる設計にする。
- 他 user resource へのアクセスは原則として存在を漏らさない。

## 非目標

- 具体的な外部 authorization provider の選定。
- UI の権限管理画面の設計。
- public share URL / publication scope の最終仕様。
- row-level security を初期実装で必須化すること。

## 推奨モデル

初期の local authorization model は `world` を認可 boundary とする。

```sql
CREATE TYPE world_member_role AS ENUM (
    'owner',
    'editor',
    'viewer'
);

CREATE TABLE world_member (
    world_id BIGINT NOT NULL REFERENCES world(world_id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    role world_member_role NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (world_id, user_id)
);

CREATE INDEX idx_world_member_user ON world_member(user_id);
```

`POST /api/worlds/create` は、作成者を `owner` として `world_member` に追加する。

短期で collaborator を実装しない場合でも、`world_member` table は 1 world につき owner 1人の形で始めてよい。将来 collaborator / shared workspace を追加する場合も同じ table を拡張できる。

## Permission 方針

初期 role semantics:

- `viewer`: read のみ。
- `editor`: diagram / entity / person / relationship の作成・更新・削除が可能。
- `owner`: world update / world delete / member 管理が可能。

初期実装で role を細かく使わない場合でも、最低限次を満たす。

- read API は `viewer` 以上を要求する。
- content write API は `editor` 以上を要求する。
- world delete と member 管理は `owner` を要求する。

## Application Boundary

認証済み user は driver layer の handler で `Extension<UserTable>` として取得し、usecase schema に `actor_user_id` を含める。

例:

```rust
pub struct GetWorldSchema {
    pub world_id: usize,
    pub actor_user_id: Uuid,
}

pub struct GetGenealogyDiagramSchema {
    pub diagram_id: usize,
    pub actor_user_id: Uuid,
}

pub struct GetGenealogyOverviewSchema {
    pub world_id: usize,
    pub actor_user_id: Uuid,
    pub diagram_ids: Option<Vec<usize>>,
    pub as_of: Option<NaiveDate>,
}
```

repository は resource ID だけで query してはならない。必ず `world_member` を join し、actor user と required role で絞る。

World read:

```sql
SELECT w.world_id, w.name, w.description, w.created_at, w.updated_at
FROM world AS w
INNER JOIN world_member AS wm
  ON wm.world_id = w.world_id
WHERE
  w.world_id = $1
  AND wm.user_id = $2
  AND w.deleted_at IS NULL;
```

Diagram read:

```sql
SELECT d.diagram_id, d.world_id, d.name, d.kind
FROM diagram AS d
INNER JOIN world_member AS wm
  ON wm.world_id = d.world_id
WHERE
  d.diagram_id = $1
  AND wm.user_id = $2
  AND d.deleted_at IS NULL;
```

Relationship / person / entity は、それぞれ `diagram` または `entity.world_id` を経由して `world_member` に到達し、同じ actor scope を適用する。

## Genealogy Overview

`GET /api/genealogy/world/{world_id}` は、まず actor が world に read access を持つことを確認する。

`diagram_ids` query が指定された場合は、query 対象を次の条件で絞る。

- `diagram.world_id = path world_id`
- actor がその world に read access を持つ
- `diagram.deleted_at IS NULL`
- `diagram.kind = 'family_tree'`
- `diagram.genealogy_overview_enabled = true`

指定された `diagram_ids` の中に actor から見えない diagram が含まれていても、他 user の diagram を response に混入させてはならない。

全ての指定 diagram が無効または不可視の場合は、既存の overview semantics に合わせて `409 CONFLICT` または `404 NOT FOUND` のどちらを返すかを implementation step で決める。ただし他 user resource の存在を示す message は返さない。

## 外部 Authorization SaaS を使う場合

将来、Permit.io、Auth0 FGA、Oso Cloud、Aserto、OpenFGA 系の外部 authorization SaaS / engine を使う可能性がある。

外部 service を使う場合でも、次の境界は application 側に残す。

- 認証済み user identity を handler から usecase へ渡す。
- usecase は resource action ごとに authorization decision を要求する。
- repository query は authorization decision 後も resource scope を絞る。
- list / overview / bulk load は actor から見える resource だけを取得する。
- 外部 service が一時的に利用できない場合の fail-closed 方針を決める。

外部 authorization は主に policy decision point として扱う。DB query の tenant / world scope filter を完全に省略して、外部 decision だけに依存してはならない。特に overview のような bulk read では、query 自体が actor scope で絞られていないと、後段 filter 漏れで情報が混入しやすい。

外部 service を使う場合の方針:

- Policy decision: 外部 authorization provider が担当してよい。
- Resource relationship source of truth: `world_member` または provider 側 tuple store のどちらを正にするかを決める。
- Query-time filtering: DB 側で actor scope を再現できる形にする。provider 側を正にする場合も、accessible world IDs を取得して query に渡すなどの boundary を作る。
- Audit: authorization decision と actor / resource / action を記録できるようにする。

## HTTP Status

推奨:

- 未認証: `401 UNAUTHORIZED`
- actor から見えない resource: 原則 `404 NOT FOUND`
- actor から見えるが action 権限が足りない resource: `403 FORBIDDEN`

resource ID の存在確認を防ぐため、read path では unauthorized と missing を区別しない。write path でも、resource が actor scope にない場合は `404` に寄せる。

## 実装方針

段階的に実装する。

1. `world_member` と role enum を追加する。
2. world create 時に creator を owner として登録する。
3. driver handler で `Extension<UserTable>` を受け取り、usecase schema に `actor_user_id` を追加する。
4. world CRUD repository query を `world_member` で scope する。
5. diagram / genealogy diagram / genealogy overview を `actor_user_id` で scope する。
6. person / relationship / entity write を `actor_user_id` と role で scope する。
7. cross-user fixture を追加し、他 user resource が `404` になることを test する。
8. 外部 authorization provider を採用する場合は、local table と provider tuple store のどちらを正にするかを別RFCまたはADRで決める。

## Test Plan

最小 coverage:

- user A が作成した world を user B が `GET /api/worlds/{world_id}` すると `404`。
- user B の `GET /api/worlds` に user A の world が含まれない。
- user A の diagram を user B が `GET /api/genealogy/diagram/{diagram_id}` すると `404`。
- user B が user A の `world_id` で `GET /api/genealogy/world/{world_id}` すると `404`。
- overview の `diagram_ids` に他 user の diagram ID を混ぜても response に含まれない。
- viewer が write API を呼ぶと `403` または `404` になる。
- editor が world delete を呼ぶと `403` になる。
- owner は world delete ができる。

## 未解決事項

- 初期実装で local `world_member` を正にするか、外部 authorization provider の tuple store を正にするか。
- collaborator / invitation API をどの endpoint と response shape で提供するか。
- public share URL / publication scope を `world_member` と同じ model に載せるか、別 model にするか。
- PostgreSQL Row Level Security を採用するか。
- authorization decision の audit log をどの粒度で保存するか。
