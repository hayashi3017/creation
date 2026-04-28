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
  ('00000000-0000-0000-0000-000000000001', 'genealogy-overview-test@example.com', 'genealogy_overview_test', 'test_password', 'default.png', 'user');

INSERT INTO world
  (world_id, name, description)
VALUES
  (1, 'Overview World', 'fixture world'),
  (2, 'Other World', 'world outside the overview');

INSERT INTO diagram
  (diagram_id, world_id, name, kind, genealogy_overview_enabled, description, deleted_at)
VALUES
  (1, 1, 'Main Line', 'family_tree', true, 'first visible diagram', NULL),
  (2, 1, 'Branch Line', 'family_tree', true, 'second visible diagram', NULL),
  (3, 1, 'Private Draft', 'family_tree', false, 'disabled for overview', NULL),
  (4, 2, 'Other World Tree', 'family_tree', true, 'outside world', NULL),
  (5, 1, 'Deleted Tree', 'family_tree', true, 'deleted diagram', now()),
  (6, 1, 'Correlation', 'correlation', true, 'wrong diagram kind', NULL);

INSERT INTO entity
  (entity_id, world_id, kind, name, description, deleted_at)
VALUES
  (1, 1, 'person', 'Shared Ancestor', 'appears in both visible diagrams', NULL),
  (2, 1, 'person', 'Child', 'deduped relationship target', NULL),
  (3, 1, 'person', 'Future Child', 'excluded by as_of birth date', NULL),
  (4, 1, 'person', 'Branch Root', 'second diagram only', NULL),
  (5, 1, 'person', 'Hidden Person', 'disabled diagram only', NULL),
  (6, 2, 'person', 'Other World Person', 'outside world', NULL),
  (7, 1, 'person', 'Deleted Entity', 'excluded endpoint', now());

INSERT INTO diagram_entity
  (diagram_id, entity_id)
VALUES
  (1, 1),
  (1, 2),
  (1, 3),
  (1, 7),
  (2, 1),
  (2, 2),
  (2, 4),
  (3, 5),
  (4, 6),
  (5, 1),
  (6, 1);

INSERT INTO person
  (entity_id, gender, birth_date, death_date, birthplace, residence, photo_url, deleted_at)
VALUES
  (1, 'female', '1950-01-01', NULL, 'Tokyo', 'Tokyo', 'https://example.com/1.png', NULL),
  (2, 'male', '1975-01-01', NULL, 'Osaka', 'Osaka', 'https://example.com/2.png', NULL),
  (3, 'unknown', '2010-01-01', NULL, 'Kyoto', NULL, NULL, NULL),
  (4, 'other', '1965-01-01', NULL, 'Nagoya', NULL, NULL, NULL),
  (5, 'female', '1980-01-01', NULL, NULL, NULL, NULL, NULL),
  (6, 'male', '1990-01-01', NULL, NULL, NULL, NULL, NULL),
  (7, 'unknown', '2000-01-01', NULL, NULL, NULL, NULL, NULL);

INSERT INTO relationship
  (relationship_id, diagram_id, source_entity_id, target_entity_id, kind, start_date, end_date, end_reason, notes, deleted_at)
VALUES
  (1, 1, 1, 2, 'parent', '1975-01-01', NULL, NULL, 'main line parent', NULL),
  (2, 2, 1, 2, 'parent', '1975-01-01', NULL, NULL, 'duplicate branch parent', NULL),
  (3, 1, 2, 3, 'parent', '2010-01-01', NULL, NULL, 'future child', NULL),
  (4, 2, 4, 2, 'adoptive_parent', '1990-01-01', '1999-12-31', 'ended', 'past relationship', NULL),
  (5, 3, 5, 1, 'parent', NULL, NULL, NULL, 'disabled diagram edge', NULL),
  (6, 4, 6, 1, 'parent', NULL, NULL, NULL, 'other world edge', NULL),
  (7, 1, 1, 7, 'parent', NULL, NULL, NULL, 'deleted endpoint edge', NULL),
  (8, 2, 1, 4, 'spouse', NULL, NULL, NULL, 'symmetric edge', NULL);
