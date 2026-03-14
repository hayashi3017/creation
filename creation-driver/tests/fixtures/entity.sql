CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE diagram_kind AS ENUM ('family_tree', 'correlation');
CREATE TYPE entity_kind AS ENUM ('person');

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

CREATE TABLE IF NOT EXISTS diagram (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    kind diagram_kind NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

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

INSERT INTO users
  (id, email, name, password, photo, role)
  VALUES
  ('00000000-0000-0000-0000-000000000001', 'entity-test@example.com', 'entity_test', 'test_password', 'default.png', 'user');

INSERT INTO diagram
  (id, name, kind, description)
  VALUES
  (1, 'Test Diagram 1', 'family_tree', 'first diagram'),
  (2, 'Test Diagram 2', 'correlation', NULL);

SELECT setval(pg_get_serial_sequence('diagram', 'id'), 2, true);

INSERT INTO entity
  (id, diagram_id, kind, name, description)
  VALUES
  (1, 1, 'person', 'Test Entity 1', 'first entity'),
  (2, 1, 'person', 'Test Entity 2', NULL),
  (3, 2, 'person', 'Other Diagram Entity', 'other diagram');

SELECT setval(pg_get_serial_sequence('entity', 'id'), 3, true);
