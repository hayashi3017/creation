-- CREATE TYPE IF NOT EXISTS が使えない環境のため
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'diagram_kind') THEN
        CREATE TYPE diagram_kind AS ENUM ('family_tree', 'correlation');
    END IF;
END
$$;

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

INSERT INTO world
  (world_id, name, description)
  VALUES
  (1, 'Repository World', 'fixture world');

SELECT setval(pg_get_serial_sequence('world', 'world_id'), 1, true);

INSERT INTO diagram
  (name, world_id, kind, description)
  VALUES
  ('Active Diagram 1', 1, 'family_tree', 'first active'),
  ('Active Diagram 2', 1, 'correlation', 'second active');

INSERT INTO diagram
  (name, world_id, kind, description, deleted_at)
  VALUES
  ('Deleted Diagram', 1, 'family_tree', 'should be filtered', now());
