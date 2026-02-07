# APIテスト導入ガイド（アーキテクチャ方針）

本ドキュメントは、本リポジトリのAPIテストを段階的に拡充するための設計方針と運用ルールをまとめたものです。現在の構成（Axum + sqlx + layered構成）に合わせ、信頼性と実行速度のバランスを取ります。

## 対象レイヤーと責務
- **Driver層（creation-driver）**: HTTP入出力の結合テスト。ルーティング、バリデーション、レスポンス形式の検証を担う。
- **Usecase/Service層**: ビジネスロジックの単体テスト。外部依存はモック化し、ドメインの条件分岐を重点的に検証。
- **Adapter層**: 永続化・リポジトリの統合テスト。sqlx::testで実DBに対してクエリの正しさを確認。

## テスト構成（推奨）
1. **API統合テスト（最優先）**
   - 配置: `creation-driver/tests/`（例: `user.rs`）
   - `sqlx::test` と fixtures を使用し、HTTP経由の振る舞いを確認。
2. **Service単体テスト**
   - 配置: `creation-service/src/service/*` の `mod tests` か `tests/`
   - 依存はtraitで分離し、モック実装を注入。
3. **Repositoryテスト**
   - 配置: `creation-adapter/tests/`（必要なら新設）
   - SQL/スキーマの互換性を重点確認。

## DB・環境依存の扱い
- `DATABASE_URL` を必須とし、テスト専用DBを使用する。
- 既存の fixtures（例: `creation-driver/tests/fixtures/user.sql`）を基準に追加。
- マイグレーションの運用は `xtask` に合わせる（`cargo run -p xtask -- migrate`）。

## 命名規約とテスト粒度
- APIテスト名は「期待する振る舞い」を主語にする（例: `duplicate_regist_email`）。
- 1テスト = 1条件の成功/失敗に絞る。
- エンドポイントの「認可」「バリデーション」「正常系」を最低限カバーする。

## 追加推奨（中期）
- テスト用ユーティリティの共通化（`creation-driver/tests/common/` に集約）。
- HTTPレスポンスのスナップショット化（例: `insta` の導入検討）。
- CIで `cargo test -p creation-driver` を必須化。
