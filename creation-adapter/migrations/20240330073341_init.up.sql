-- type
CREATE TYPE diagram_kind AS ENUM ('family_tree', 'correlation');
CREATE TYPE entity_kind AS ENUM ('person');
CREATE TYPE gender_kind AS ENUM ('male', 'female', 'other', 'unknown');
CREATE TYPE relationship_kind AS ENUM (
  'parent',  -- 親
  'child', -- 子
  'sibling', -- 兄弟姉妹
  'spouse', -- 配偶者
  'adopted_parent', -- 養親
  'adopted_child', -- 養子
  'divorced_spouse', -- 離婚した配偶者
  'cohabitant', -- 同居人
  'step_parent', -- 継親
  'step_child' -- 継子
);



-- 必要な拡張（UUID v4用）
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users テーブル
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    photo VARCHAR(255) NOT NULL DEFAULT 'default.png',
    password VARCHAR(100) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS users_email_idx ON users (email);

-- Diagram テーブル
CREATE TABLE IF NOT EXISTS diagram (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    kind diagram_kind NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

-- Entity テーブル
CREATE TABLE IF NOT EXISTS entity (
    id BIGSERIAL PRIMARY KEY,
    diagram_id BIGINT NOT NULL REFERENCES diagram(id) ON DELETE CASCADE,
    kind entity_kind NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

-- Person テーブル
CREATE TABLE IF NOT EXISTS person (
    entity_id BIGINT PRIMARY KEY REFERENCES entity(id) ON DELETE CASCADE,
    gender gender_kind DEFAULT 'unknown',
    birth_date DATE,
    death_date DATE,
    birthplace VARCHAR(255),
    residence VARCHAR(255),
    photo_url VARCHAR(512),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

-- Relationship テーブル
CREATE TABLE IF NOT EXISTS relationship (
    id BIGSERIAL PRIMARY KEY,
    diagram_id BIGINT NOT NULL REFERENCES diagram(id) ON DELETE CASCADE,
    source_entity_id BIGINT NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    target_entity_id BIGINT NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    kind relationship_kind NOT NULL,
    start_date DATE,
    end_date DATE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

-- TreePath テーブル（閉包テーブルパターン用）
CREATE TABLE IF NOT EXISTS tree_path (
    ancestor_id BIGINT NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    descendant_id BIGINT NOT NULL REFERENCES entity(id) ON DELETE CASCADE,
    depth INT NOT NULL,
    PRIMARY KEY (ancestor_id, descendant_id)
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_entity_kind ON entity(kind);
CREATE INDEX IF NOT EXISTS idx_relationship_source_entity ON relationship(source_entity_id);
CREATE INDEX IF NOT EXISTS idx_relationship_target_entity ON relationship(target_entity_id);
CREATE INDEX IF NOT EXISTS idx_relationship_kind ON relationship(kind);
CREATE INDEX IF NOT EXISTS idx_entity_diagram ON entity(diagram_id);
CREATE INDEX IF NOT EXISTS idx_relationship_diagram ON relationship(diagram_id);
CREATE INDEX IF NOT EXISTS idx_tree_path_ancestor ON tree_path(ancestor_id);
CREATE INDEX IF NOT EXISTS idx_tree_path_descendant ON tree_path(descendant_id);
