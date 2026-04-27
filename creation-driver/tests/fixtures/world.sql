CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE diagram_kind AS ENUM ('family_tree', 'correlation');
CREATE TYPE entity_kind AS ENUM ('person');
CREATE TYPE gender_kind AS ENUM ('male', 'female', 'other', 'unknown');
CREATE TYPE relationship_kind AS ENUM (
    'parent',
    'adoptive_parent',
    'step_parent',
    'spouse',
    'partner',
    'cohabitant'
);

CREATE TABLE IF NOT EXISTS users (
    user_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    photo VARCHAR(255) NOT NULL DEFAULT 'default.png',
    password VARCHAR(100) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS world (
    world_id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS diagram (
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

CREATE TABLE IF NOT EXISTS entity (
    entity_id BIGSERIAL PRIMARY KEY,
    world_id BIGINT NOT NULL REFERENCES world(world_id) ON DELETE CASCADE,
    kind entity_kind NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS diagram_entity (
    diagram_id BIGINT NOT NULL REFERENCES diagram(diagram_id) ON DELETE CASCADE,
    entity_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    PRIMARY KEY (diagram_id, entity_id)
);

CREATE TABLE IF NOT EXISTS person (
    entity_id BIGINT PRIMARY KEY REFERENCES entity(entity_id) ON DELETE CASCADE,
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

CREATE TABLE IF NOT EXISTS relationship (
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
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS tree_path (
    ancestor_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    descendant_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    depth INT NOT NULL,
    PRIMARY KEY (ancestor_id, descendant_id)
);

INSERT INTO users
  (user_id, email, name, password, photo, role)
  VALUES
  ('00000000-0000-0000-0000-000000000001', 'world-test@example.com', 'world_test', 'test_password', 'default.png', 'user');

INSERT INTO world
  (world_id, name, description, updated_at, deleted_at)
  VALUES
  (1, 'Active World', 'active fixture', now() - interval '1 hour', NULL),
  (2, 'Second World', NULL, now(), NULL),
  (3, 'Deleted World', 'deleted fixture', now() - interval '2 hours', now());

SELECT setval(pg_get_serial_sequence('world', 'world_id'), 3, true);

INSERT INTO diagram
  (diagram_id, world_id, name, kind)
  VALUES
  (1, 1, 'Main Diagram', 'family_tree'),
  (2, 1, 'Side Diagram', 'family_tree');

INSERT INTO entity
  (entity_id, world_id, kind, name)
  VALUES
  (1, 1, 'person', 'Parent'),
  (2, 1, 'person', 'Child');

INSERT INTO person
  (entity_id, gender)
  VALUES
  (1, 'unknown'),
  (2, 'unknown');

INSERT INTO diagram_entity
  (diagram_id, entity_id)
  VALUES
  (1, 1),
  (1, 2);

INSERT INTO relationship
  (relationship_id, diagram_id, source_entity_id, target_entity_id, kind)
  VALUES
  (1, 1, 1, 2, 'parent');

INSERT INTO tree_path
  (ancestor_id, descendant_id, depth)
  VALUES
  (1, 2, 1);
