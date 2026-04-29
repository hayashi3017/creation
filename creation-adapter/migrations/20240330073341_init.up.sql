CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE diagram_kind AS ENUM (
    'family_tree',
    'correlation'
);

CREATE TYPE entity_kind AS ENUM (
    'person'
);

CREATE TYPE gender_kind AS ENUM (
    'male',
    'female',
    'other',
    'unknown'
);

CREATE TYPE relationship_kind AS ENUM (
    'parent',
    'adoptive_parent',
    'step_parent',
    'spouse',
    'partner',
    'cohabitant'
);

CREATE TABLE users (
    user_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    photo VARCHAR(255) NOT NULL DEFAULT 'default.png',
    password VARCHAR(100) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

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
    genealogy_overview_enabled BOOLEAN NOT NULL DEFAULT true,
    description TEXT,
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

CREATE TABLE diagram_entity (
    diagram_id BIGINT NOT NULL REFERENCES diagram(diagram_id) ON DELETE CASCADE,
    entity_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    PRIMARY KEY (diagram_id, entity_id)
);

CREATE TABLE person (
    entity_id BIGINT PRIMARY KEY REFERENCES entity(entity_id) ON DELETE CASCADE,
    first_name VARCHAR(255),
    middle_name VARCHAR(255),
    last_name VARCHAR(255),
    first_name_kana VARCHAR(255),
    middle_name_kana VARCHAR(255),
    last_name_kana VARCHAR(255),
    first_name_romaji VARCHAR(255),
    middle_name_romaji VARCHAR(255),
    last_name_romaji VARCHAR(255),
    gender gender_kind DEFAULT 'unknown',
    birth_date DATE,
    death_date DATE,
    birthplace VARCHAR(255),
    deathplace VARCHAR(255),
    residence VARCHAR(255),
    photo_url VARCHAR(512),
    profile_text TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE relationship (
    relationship_id BIGSERIAL PRIMARY KEY,
    diagram_id BIGINT NOT NULL REFERENCES diagram(diagram_id) ON DELETE CASCADE,
    source_entity_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    target_entity_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    kind relationship_kind NOT NULL,
    start_date DATE,
    end_date DATE,
    end_reason VARCHAR(32),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT chk_relationship_no_active_self_relation
        CHECK (deleted_at IS NOT NULL OR source_entity_id <> target_entity_id)
);

CREATE TABLE tree_path (
    ancestor_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    descendant_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    depth INT NOT NULL,
    PRIMARY KEY (ancestor_id, descendant_id)
);

CREATE INDEX users_email_idx ON users (email);
CREATE INDEX idx_diagram_world ON diagram(world_id);
CREATE INDEX idx_entity_kind ON entity(kind);
CREATE INDEX idx_entity_world ON entity(world_id);
CREATE INDEX idx_diagram_entity_diagram ON diagram_entity(diagram_id);
CREATE INDEX idx_diagram_entity_entity ON diagram_entity(entity_id);
CREATE INDEX idx_person_last_name ON person(last_name);
CREATE INDEX idx_person_first_name ON person(first_name);
CREATE INDEX idx_person_last_name_kana ON person(last_name_kana);
CREATE INDEX idx_person_last_name_romaji ON person(last_name_romaji);
CREATE INDEX idx_person_birth_date ON person(birth_date);
CREATE INDEX idx_person_death_date ON person(death_date);
CREATE INDEX idx_relationship_source_entity ON relationship(source_entity_id);
CREATE INDEX idx_relationship_target_entity ON relationship(target_entity_id);
CREATE INDEX idx_relationship_kind ON relationship(kind);
CREATE INDEX idx_relationship_diagram ON relationship(diagram_id);
CREATE INDEX idx_tree_path_ancestor ON tree_path(ancestor_id);
CREATE INDEX idx_tree_path_descendant ON tree_path(descendant_id);

CREATE UNIQUE INDEX uq_relationship_directed_active
ON relationship (
    diagram_id,
    source_entity_id,
    target_entity_id,
    kind
)
WHERE deleted_at IS NULL
    AND kind IN ('parent', 'adoptive_parent', 'step_parent');

CREATE UNIQUE INDEX uq_relationship_symmetric_active
ON relationship (
    diagram_id,
    LEAST(source_entity_id, target_entity_id),
    GREATEST(source_entity_id, target_entity_id),
    kind
)
WHERE deleted_at IS NULL
    AND kind IN ('spouse', 'partner', 'cohabitant');
