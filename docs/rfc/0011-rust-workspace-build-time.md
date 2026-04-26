# RFC 0011: Rust ワークスペースのローカルビルド時間ポリシー

- 状態: `下書き`
- 最終更新: `2026-04-07`

## 背景

Rust ワークスペースが大きくなるほど、ローカルのフィードバックループは設定に左右されやすくなる。

- 複数 crate にまたがるワークスペース全体の再コンパイルが連鎖する
- build script が必要以上に再実行される可能性がある
- `cargo check` で十分な場面でも `cargo build` が使われがちである
- 日常的な確認には `release` 向け設定が重い
- ワークスペース全体のビルドキャッシュ設定がまだ標準化されていない

リポジトリルートには Cargo workspace があるが、現状では次を定義していない。

- 明示的なワークスペースレベルの `profile.dev`
- `dev` と `release` の中間となる高速な検証用 profile
- `.cargo/config.toml` によるワークスペースレベルの `sccache` 連携
- `default-members` の運用ポリシー
- `build.rs` の再実行条件に関する運用ポリシー

執筆時点では、ルート [Cargo.toml](/home/hayashi3017/git/creation/Cargo.toml) に `[workspace]` と `[workspace.dependencies]` はあるが `[profile.*]` はなく、リポジトリの `.cargo/config.toml` も `build.rs` も存在しない。

## 目標

本番用の最終 `release` profile を弱めずに、再現可能でワークスペース全体に効く形でローカル開発ビルド時間を短縮する。

## 非目標

- 本番実行時性能の最大化
- CI の全面的な再設計
- linker 固有または target 固有の調整
- 大規模な依存関係削減
- 広範な feature flag 再構成

これらは有用な可能性があるが、別作業として扱う。

## 提案

このワークスペースに次のローカルビルドポリシーを導入する。

1. build profile はルート `Cargo.toml` にのみ定義する。
2. `profile.dev` は edit-compile-check ループ向けに明示的に最適化する。
3. 高速なローカル最適化ビルド用に `release-fast` profile を追加する。
4. `.cargo/config.toml` による任意のワークスペースレベル `sccache` 対応を追加する。
5. `workspace.default-members` で既定ビルド範囲を狭めるべきか確認する。
6. 将来の `build.rs` には明示的な `rerun-if-*` ルールを求める。
7. 日常的な確認コマンドの既定を `cargo check` として文書化する。
8. 遅いビルドの調査には `cargo --timings` を使う。

## 現在の状態

- ルート [Cargo.toml](/home/hayashi3017/git/creation/Cargo.toml) に `[profile.dev]` はない。
- `release-fast` のようなカスタム release 系 profile はない。
- `default-members` はない。
- リポジトリの `.cargo/config.toml` はない。
- 現在 `build.rs` は存在しない。

したがって、この RFC は主にワークスペースがさらに大きくなる前にポリシーと基準を整えるためのものである。

## Profile ポリシー

### ルートのみで定義

Build profile はワークスペースルートの [Cargo.toml](/home/hayashi3017/git/creation/Cargo.toml) にのみ定義する。

理由:

- Cargo profile の挙動は、1 つのファイルがポリシーを所有した方が分かりやすい。
- member crate がローカルビルド挙動を暗黙に分岐させるべきではない。
- 将来のレビューを小さく集約できる。

### `profile.dev`

推奨する基準:

```toml
[profile.dev]
incremental = true
debug = "line-tables-only"
```

意図:

- 頻繁な編集に対して incremental compilation を有効に保つ。
- 利用可能な backtrace を残しつつ debug info 生成コストを下げる。

利用中の toolchain で `debug = "line-tables-only"` が使えない場合は、`debug = 1` を優先する。

`dev` では次を標準化しない。

- カスタム `opt-level`
- `lto`
- 攻めた `codegen-units` 調整

既定の開発 profile は benchmark 的な挙動ではなく、高速な反復に最適化する。

### `profile.release-fast`

ローカル最適化検証用 profile を追加する。

```toml
[profile.release-fast]
inherits = "release"
incremental = true
lto = "off"
codegen-units = 64
debug = "line-tables-only"
```

意図:

- 完全な `release` より速い代替を用意する。
- 本番 `release` は変更しない。
- 完全な release ビルドのコストを払わず、概ね最適化された挙動をローカルで確認できるようにする。

用途:

- 最適化下での簡易 smoke test
- binary size や速度の概算確認
- release でのみ出る挙動のローカル再現

## Cache ポリシー

既存 tooling と衝突しない場合、`.cargo/config.toml` に次を追加する。

```toml
[build]
rustc-workspace-wrapper = "sccache"
```

意図:

- crate 間の再利用を改善する。
- branch 切り替えや依存変更後の再ビルドコストを下げる。

制約:

- `sccache` は必須要件ではなく任意にする。
- wrapper 設定が既にある場合は互換性を確認してから変更する。
- 環境回避策を入れる場合は、その近くに短い理由コメントを書く。

## ワークスペース範囲ポリシー

日常開発で既定の対象が本当に全 member である必要があるか確認する。

将来案:

```toml
[workspace]
members = ["creation-adapter", "creation-driver", "creation-service", "creation-usecase", "xtask"]
default-members = ["creation-driver", "creation-adapter", "creation-service", "creation-usecase"]
```

これはレビュー対象のポリシー提案であり、自動的な決定ではない。`default-members` は実際の開発利用と一致する場合にだけ使う。狭めすぎると contributor を混乱させ、利用頻度の低い crate の破損を隠す。

## Build Script ポリシー

現在 `build.rs` はないが、方針は先に定める。

将来の `build.rs` は、実用的な範囲で明示的な再実行条件を宣言する。

```rust
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
```

```rust
fn main() {
    println!("cargo:rerun-if-changed=schema/openapi.yaml");
    println!("cargo:rerun-if-env-changed=MY_CODEGEN_MODE");
}
```

意図:

- 不要な再実行を避ける。
- code generation の入力を明示する。
- 将来のビルド時間コストを推論しやすくする。

真の入力が分からない場合は再実行条件を狭めない。誤って小さすぎる再実行範囲は、保守的な再実行より悪い。

## コマンドポリシー

日常のローカル開発では次を優先する。

1. `cargo check`
2. binary や artifact が必要な場合だけ `cargo build`
3. 高速な最適化検証には `cargo build --profile release-fast`

```bash
cargo check --workspace
cargo check -p creation-driver
cargo build -p creation-driver --profile release-fast
```

理由:

- `cargo check` は不要な code generation を避け、フィードバックを速くする。
- 出力 artifact が必要なときだけ完全な build コストを払うべきである。

## 計測ポリシー

ビルドが遅いと感じた場合は推測せず timings を使う。

```bash
cargo build --workspace --profile release-fast --timings
```

これにより、遅い crate、高コストな build script、想定外に広い再ビルド範囲を特定する。

## 検討した代替案

### `profile.release` を直接軽くする

却下理由:

- 本番 profile の意味を弱める。
- ローカル利便性を release 期待値に結合してしまう。

### `dev` の `opt-level` を上げる

既定としては却下した。

- コンパイルを遅くすることが多い。
- 高速な反復という主目的と衝突する。

### 攻めた linker または `RUSTFLAGS` 調整

延期理由:

- 環境依存性が高い。
- contributor 間で再現可能に保ちにくい。
- トラブルシュートが複雑になる。

## 展開計画

1. ルートワークスペースの現在のビルド設定を確認する。
2. ルートに `profile.dev` を追加する。
3. ルートに `profile.release-fast` を追加する。
4. `.cargo/config.toml` と任意の `sccache` 対応を評価する。
5. `default-members` が現行 workflow に役立つか害になるか確認する。
6. developer 向け docs にコマンド指針を記録する。
7. `cargo --timings` で計測する。

## 利点

- ローカル再ビルドが速くなる。
- ビルド設定の所有箇所が明確になる。
- ローカル検証と本番 release を分離できる。
- crate 間で build policy が意図せずずれるリスクが下がる。
- crate や code generation が増える前の良い基準になる。

## 欠点

- 保守すべき明示的な Cargo 設定が増える。
- `release-fast` では完全な `release` でだけ出る問題を隠す可能性がある。
- `sccache` を採用すると説明・トラブルシュート対象の tool が増える。
- `default-members` が共通 workflow とずれると contributor を混乱させる。

## レビュー観点

- このリポジトリで今 `default-members` を採用すべきか、明確な痛みが出るまで待つべきか。
- `sccache` 対応をリポジトリに commit すべきか、まず任意の local setup として文書化すべきか。
- 将来 RFC で `nextest`、scoped test workflow、fixture DB 起動コストなど test 実行遅延も扱うべきか。
