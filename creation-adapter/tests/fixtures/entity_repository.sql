-- CREATE TYPE IF NOT EXISTS が使えない環境のため
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'diagram_kind') THEN
        CREATE TYPE diagram_kind AS ENUM ('family_tree', 'correlation');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'entity_kind') THEN
        CREATE TYPE entity_kind AS ENUM ('person');
    END IF;
END
$$;

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

INSERT INTO diagram
  (id, name, kind, description)
  VALUES
  (1, 'Repository Diagram 1', 'family_tree', 'first diagram'),
  (2, 'Repository Diagram 2', 'correlation', 'second diagram');

SELECT setval(pg_get_serial_sequence('diagram', 'id'), 2, true);

INSERT INTO entity
  (id, diagram_id, kind, name, description)
  VALUES
  (1, 1, 'person', 'Active Entity 1', 'first active'),
  (2, 1, 'person', 'Active Entity 2', 'second active'),
  (4, 2, 'person', 'Other Diagram Entity', 'belongs to another diagram');

INSERT INTO entity
  (id, diagram_id, kind, name, description, deleted_at)
  VALUES
  (3, 1, 'person', 'Deleted Entity', 'should be filtered', now());

SELECT setval(pg_get_serial_sequence('entity', 'id'), 4, true);
