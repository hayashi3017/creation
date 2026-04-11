CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE diagram_kind AS ENUM ('family_tree', 'correlation');
CREATE TYPE entity_kind AS ENUM ('person');
CREATE TYPE gender_kind AS ENUM ('male', 'female', 'other', 'unknown');
CREATE TYPE relationship_kind AS ENUM (
    'parent',
    'child',
    'sibling',
    'spouse',
    'adopted_parent',
    'adopted_child',
    'divorced_spouse',
    'cohabitant',
    'step_parent',
    'step_child'
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
  ('00000000-0000-0000-0000-000000000001', 'person-test@example.com', 'person_test', 'test_password', 'default.png', 'user');

INSERT INTO diagram
  (diagram_id, name, kind, description)
  VALUES
  (1, 'Test Diagram 1', 'family_tree', 'first diagram'),
  (2, 'Test Diagram 2', 'correlation', 'second diagram');

SELECT setval(pg_get_serial_sequence('diagram', 'diagram_id'), 2, true);

INSERT INTO entity
  (entity_id, diagram_id, kind, name, description, deleted_at)
  VALUES
  (1, 1, 'person', 'Test Person 1', 'first person', NULL),
  (2, 1, 'person', 'Test Person 2', NULL, NULL),
  (3, 2, 'person', 'Other Diagram Person', 'other diagram', NULL),
  (4, 1, 'person', 'Deleted Entity Person', 'deleted entity', now()),
  (5, 1, 'person', 'Deleted Person Row', 'deleted person', NULL);

SELECT setval(pg_get_serial_sequence('entity', 'entity_id'), 5, true);

INSERT INTO person
  (entity_id, gender, birth_date, death_date, birthplace, residence, photo_url, deleted_at)
  VALUES
  (1, 'female', '1995-03-10', NULL, 'Tokyo', 'Nagoya', 'https://example.com/person-1.png', NULL),
  (2, 'male', NULL, NULL, 'Kyoto', NULL, NULL, NULL),
  (3, 'other', NULL, NULL, 'Osaka', NULL, NULL, NULL),
  (4, 'unknown', NULL, NULL, 'Sapporo', NULL, NULL, NULL),
  (5, 'female', NULL, NULL, 'Fukuoka', NULL, NULL, now());

INSERT INTO relationship
  (relationship_id, diagram_id, source_entity_id, target_entity_id, kind, notes, deleted_at)
  VALUES
  (1, 1, 1, 2, 'parent', 'person fixture lineage', NULL);

SELECT setval(pg_get_serial_sequence('relationship', 'relationship_id'), 1, true);

INSERT INTO tree_path
  (ancestor_id, descendant_id, depth)
  VALUES
  (1, 1, 0),
  (1, 2, 1),
  (2, 2, 0),
  (3, 3, 0),
  (5, 5, 0);
