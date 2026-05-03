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
  ('00000000-0000-0000-0000-000000000001', 'family-tree-test@example.com', 'family_tree_test', 'test_password', 'default.png', 'user');

INSERT INTO world
  (world_id, name, description)
VALUES
  (1, 'Family Tree Test World', 'fixture world');

SELECT setval(pg_get_serial_sequence('world', 'world_id'), 1, true);

INSERT INTO diagram
  (diagram_id, world_id, name, kind, description, deleted_at)
VALUES
  (1, 1, 'Family Tree Diagram', 'family_tree', 'normalized family tree', NULL),
  (2, 1, 'Correlation Diagram', 'correlation', 'non family tree', NULL),
  (3, 1, 'Deleted Family Tree', 'family_tree', 'soft deleted diagram', now());

SELECT setval(pg_get_serial_sequence('diagram', 'diagram_id'), 3, true);

INSERT INTO entity
  (entity_id, world_id, kind, name, description, deleted_at)
VALUES
  (1, 1, 'person', 'Ancestor', 'root of first branch', NULL),
  (2, 1, 'person', 'Parent', 'middle generation', NULL),
  (3, 1, 'person', 'Child', 'leaf node', NULL),
  (4, 1, 'person', 'Other Root', 'root of second branch', NULL),
  (5, 1, 'person', 'Other Root Child', 'second branch leaf', NULL),
  (6, 1, 'person', 'Deleted Endpoint', 'should not appear', now()),
  (7, 1, 'person', 'Correlation Person', 'wrong kind', NULL),
  (8, 1, 'person', 'Deleted Diagram Parent', 'soft deleted diagram', NULL),
  (9, 1, 'person', 'Deleted Diagram Child', 'soft deleted diagram', NULL);

SELECT setval(pg_get_serial_sequence('entity', 'entity_id'), 9, true);

INSERT INTO diagram_entity
  (diagram_id, entity_id)
VALUES
  (1, 1),
  (1, 2),
  (1, 3),
  (1, 4),
  (1, 5),
  (1, 6),
  (2, 7),
  (3, 8),
  (3, 9);

INSERT INTO person
  (entity_id, gender, birth_date, death_date, birthplace, residence, photo_url, deleted_at)
VALUES
  (1, 'female', '1950-01-01', NULL, 'Tokyo', 'Tokyo', 'https://example.com/1.png', NULL),
  (2, 'male', '1975-05-05', NULL, 'Osaka', 'Nagoya', 'https://example.com/2.png', NULL),
  (3, 'other', '2000-09-09', NULL, 'Kyoto', 'Kyoto', 'https://example.com/3.png', NULL),
  (4, 'female', '1960-03-03', NULL, 'Sapporo', 'Sapporo', NULL, NULL),
  (5, 'male', '1990-07-07', NULL, 'Fukuoka', 'Fukuoka', NULL, NULL),
  (6, 'unknown', NULL, NULL, 'Nagasaki', NULL, NULL, NULL),
  (7, 'female', NULL, NULL, 'Kobe', NULL, NULL, NULL),
  (8, 'male', NULL, NULL, 'Yokohama', NULL, NULL, NULL),
  (9, 'female', NULL, NULL, 'Okinawa', NULL, NULL, NULL);

INSERT INTO relationship
  (relationship_id, diagram_id, source_entity_id, target_entity_id, kind, start_date, end_date, notes, deleted_at)
VALUES
  (1, 1, 1, 2, 'parent', NULL, NULL, 'ancestor to parent', NULL),
  (2, 1, 2, 3, 'parent', '2000-09-09', NULL, 'parent to child', NULL),
  (3, 1, 4, 5, 'parent', '1990-07-07', '1999-12-31', 'second branch', NULL),
  (4, 1, 1, 6, 'parent', NULL, NULL, 'deleted endpoint edge', NULL),
  (5, 1, 2, 5, 'spouse', NULL, NULL, 'non lineage edge', NULL),
  (6, 3, 8, 9, 'parent', NULL, NULL, 'deleted diagram edge', NULL);

SELECT setval(pg_get_serial_sequence('relationship', 'relationship_id'), 6, true);
