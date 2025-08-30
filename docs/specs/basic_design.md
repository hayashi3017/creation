- [🏗 基本設計書（外部設計）](#-基本設計書外部設計)
  - [1. 文書情報](#1-文書情報)
  - [2. 背景・目的](#2-背景目的)
  - [3. システム全体構成](#3-システム全体構成)
    - [3.1 システム構成図](#31-システム構成図)
    - [3.2 モジュール構成](#32-モジュール構成)
  - [4. 画面設計](#4-画面設計)
    - [4.1 画面一覧](#41-画面一覧)
    - [4.2 画面遷移図](#42-画面遷移図)
    - [4.3 各画面のレイアウト・UI仕様](#43-各画面のレイアウトui仕様)
  - [5. 機能設計](#5-機能設計)
    - [5.1 機能構成図](#51-機能構成図)
    - [5.2 機能詳細](#52-機能詳細)
  - [6. データ設計](#6-データ設計)
    - [6.1 論理データモデル（ER図）](#61-論理データモデルer図)
    - [6.2 データ項目定義](#62-データ項目定義)
      - [usersテーブル](#usersテーブル)
    - [Entity](#entity)
      - [relationshipテーブル](#relationshipテーブル)
      - [Personテーブル](#personテーブル)
  - [7. 外部インターフェース設計](#7-外部インターフェース設計)
    - [7.1 API一覧](#71-api一覧)
    - [7.2 外部システム連携](#72-外部システム連携)
  - [8. 非機能設計](#8-非機能設計)
  - [9. エラーハンドリング設計](#9-エラーハンドリング設計)
  - [10. 制約条件](#10-制約条件)
  - [11. テスト計画（概要）](#11-テスト計画概要)
  - [12. 承認・変更管理](#12-承認変更管理)
  - [付録](#付録)


# 🏗 基本設計書（外部設計）

## 1. 文書情報
| 項目       | 内容                         |
| ---------- | ---------------------------- |
| 文書名     | 相関図管理システム基本設計書 |
| 作成日     | 2025/8/11                    |
| バージョン | v0.1                         |
| 作成者     | <担当者名>                   |
| 承認者     | <承認者名>                   |

---

## 2. 背景・目的
- **背景**  
  <要件定義で決まった内容を踏まえ、なぜ本設計が必要かを記載>
- **目的**  
  <本設計で明確にする内容：画面、データ、機能構成、外部I/Fなど>

---

## 3. システム全体構成
### 3.1 システム構成図
- システム全体の構成図（アーキテクチャ図）
- クラウド・オンプレ・ハイブリッド構成の明記

### 3.2 モジュール構成
| モジュール名         | 概要                   | 担当     |
| -------------------- | ---------------------- | -------- |
| 認証モジュール       | ユーザ認証・認可を行う | <担当者> |
| 相関図管理モジュール | 商品情報の登録・更新   | <担当者> |

---

## 4. 画面設計
### 4.1 画面一覧
| No   | 画面ID | 画面名       | 概要                     |
| ---- | ------ | ------------ | ------------------------ |
| G-01 | login  | ログイン画面 | ユーザIDとパスワード入力 |

### 4.2 画面遷移図
- 画面間の遷移フロー図を記載（PlantUMLやFigmaで作成可）

### 4.3 各画面のレイアウト・UI仕様
- ワイヤーフレーム画像（添付）
- 項目名、型、必須/任意、バリデーション条件

---

## 5. 機能設計
### 5.1 機能構成図
- 機能階層図（ツリー構造）

### 5.2 機能詳細
| 機能ID | 機能名     | 入力       | 処理               | 出力            |
| ------ | ---------- | ---------- | ------------------ | --------------- |
| F-01   | ユーザ登録 | ユーザ情報 | 入力値検証、DB登録 | 成功/エラー応答 |

---

## 6. データ設計
### 6.1 論理データモデル（ER図）

```marmaid
erDiagram
    Entity {
        uuid id PK
        string type "ENUM: person, organization, event, ..."
        string name
        string description
        datetime created_at
        datetime updated_at
    }

    Person {
        uuid entity_id PK, FK
        string gender "ENUM: male, female, other, unknown"
        date birth_date
        date death_date
        string birthplace
        string residence
        string photo_url
    }

    Relationship {
        uuid id PK
        uuid entity_id_1 FK
        uuid entity_id_2 FK
        string type "ENUM: parent, spouse, member_of, located_in, ..."
        date start_date
        date end_date
        string notes
        datetime created_at
        datetime updated_at
    }

    Entity ||--|| Person : "is a"
    Entity ||--o{ Relationship : "entity_id_1"
    Entity ||--o{ Relationship : "entity_id_2"

```


### 6.2 データ項目定義

#### usersテーブル
WIP

| 項目名  | 型  | 桁数 | 必須 | 説明     |
| ------- | --- | ---- | ---- | -------- |
| user_id | int | 10   | 必須 | ユーザID |

### Entity

| カラム名     | 型             | 制約 | 説明 |
|--------------|----------------|------|------|
| id           | BIGINT     | PK, NOT NULL, DEFAULT AUTO_INCREMENT  | エンティティID |
| type         | TEXT           | NOT NULL | 種別（例: person, organization, event） |
| name         | TEXT           | NOT NULL | 名称 |
| description  | TEXT           |        | 概要・説明 |
| created_at   | DATETIME(6) | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at   | DATETIME(6) | NOT NULL, DEFAULT now() | 更新日時 |
| deleted_at   | DATETIME(6) | DEFAULT NULL | 削除日時 |

#### relationshipテーブル

| カラム名     | 型             | 制約 | 説明 |
|--------------|----------------|------|------|
| id           | BIGINT           | PK, NOT NULL, DEFAULT AUTO_INCREMENT  | 関係ID |
| entity_id_1  | UUID           | NOT NULL, FK -> entity(id) | 関係元のEntity |
| entity_id_2  | UUID           | NOT NULL, FK -> entity(id) | 関係先のEntity |
| type         | TEXT           | NOT NULL | 関係種別（例: parent, spouse, member_of, located_in） |
| start_date   | DATE           |        | 関係開始日 |
| end_date     | DATE           |        | 関係終了日 |
| notes        | TEXT           |        | メモ |
| created_at   | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT now() | 作成日時 |
| updated_at   | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT now() | 更新日時 |


#### Personテーブル

| カラム名     | 型             | 制約 | 説明 |
|--------------|----------------|------|------|
| entity_id    | BIGINT           | PK, NOT NULL, FK -> entity(id) | EntityのID |
| gender       | TEXT           |        | 性別（male, female, other, unknown など） |
| birth_date   | DATE           |        | 生年月日 |
| death_date   | DATE           |        | 没年月日 |
| birthplace   | TEXT           |        | 出生地 |
| residence    | TEXT           |        | 居住地 |
| photo_url    | TEXT           |        | 写真URL |

---

## 7. 外部インターフェース設計
### 7.1 API一覧
| API名         | HTTPメソッド | URI        | 入力 | 出力 | 概要           |
| ------------- | ------------ | ---------- | ---- | ---- | -------------- |
| ユーザ登録API | POST         | /api/users | JSON | JSON | 新規ユーザ作成 |

### 7.2 外部システム連携
- 接続方式（REST、gRPC、SOAPなど）
- 認証方式（OAuth2、APIキー）
- データ形式（JSON、CSV、XML）

---

## 8. 非機能設計
- 性能設計（レスポンス時間、スループット）
- セキュリティ設計（暗号化方式、アクセス制御）
- 運用監視項目（CPU、メモリ、ログ監視）

---

## 9. エラーハンドリング設計
| エラーコード | メッセージ            | 対応方法                       |
| ------------ | --------------------- | ------------------------------ |
| 400          | Bad Request           | 入力エラーを返却               |
| 500          | Internal Server Error | ログ出力後、汎用エラー画面表示 |

---

## 10. 制約条件
- 使用技術（言語、フレームワーク、DB）
- 利用可能なライブラリ・サービス
- インフラ条件（AWS、GCPなど）

---

## 11. テスト計画（概要）
- 単体テスト範囲
- 結合テスト範囲
- 性能テスト概要

---

## 12. 承認・変更管理
- 本設計書承認者
- 変更管理手順
- バージョン管理ルール

---

## 付録
- 用語集
- 参照資料（要件定義書リンク、関連設計資料）
