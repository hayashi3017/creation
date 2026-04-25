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

CREATE TABLE IF NOT EXISTS diagram (
    diagram_id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    kind diagram_kind NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS entity (
    entity_id BIGSERIAL PRIMARY KEY,
    diagram_id BIGINT NOT NULL REFERENCES diagram(diagram_id) ON DELETE CASCADE,
    kind entity_kind NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
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
  ('00000000-0000-0000-0000-000000000001', 'family-tree-test@example.com', 'family_tree_test', 'test_password', 'default.png', 'user');

INSERT INTO diagram
  (diagram_id, name, kind, description, deleted_at)
VALUES
  (1, 'Family Tree Diagram', 'family_tree', 'normalized family tree', NULL),
  (2, 'Correlation Diagram', 'correlation', 'non family tree', NULL),
  (3, 'Deleted Family Tree', 'family_tree', 'soft deleted diagram', now());

SELECT setval(pg_get_serial_sequence('diagram', 'diagram_id'), 3, true);

INSERT INTO entity
  (entity_id, diagram_id, kind, name, description, deleted_at)
VALUES
  (1, 1, 'person', 'Ancestor', 'root of first branch', NULL),
  (2, 1, 'person', 'Parent', 'middle generation', NULL),
  (3, 1, 'person', 'Child', 'leaf node', NULL),
  (4, 1, 'person', 'Other Root', 'root of second branch', NULL),
  (5, 1, 'person', 'Other Root Child', 'second branch leaf', NULL),
  (6, 1, 'person', 'Deleted Endpoint', 'should not appear', now()),
  (7, 2, 'person', 'Correlation Person', 'wrong kind', NULL),
  (8, 3, 'person', 'Deleted Diagram Parent', 'soft deleted diagram', NULL),
  (9, 3, 'person', 'Deleted Diagram Child', 'soft deleted diagram', NULL);

SELECT setval(pg_get_serial_sequence('entity', 'entity_id'), 9, true);

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
  (relationship_id, diagram_id, source_entity_id, target_entity_id, kind, notes, deleted_at)
VALUES
  (1, 1, 1, 2, 'parent', 'ancestor to parent', NULL),
  (2, 1, 2, 3, 'parent', 'parent to child', NULL),
  (3, 1, 4, 5, 'parent', 'second branch', NULL),
  (4, 1, 1, 6, 'parent', 'deleted endpoint edge', NULL),
  (5, 1, 2, 5, 'spouse', 'non lineage edge', NULL),
  (6, 3, 8, 9, 'parent', 'deleted diagram edge', NULL);

SELECT setval(pg_get_serial_sequence('relationship', 'relationship_id'), 6, true);
