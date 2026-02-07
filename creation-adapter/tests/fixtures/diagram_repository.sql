-- CREATE TYPE IF NOT EXISTS が使えない環境のため
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'diagram_kind') THEN
        CREATE TYPE diagram_kind AS ENUM ('family_tree', 'correlation');
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

INSERT INTO diagram
  (name, kind, description)
  VALUES
  ('Active Diagram 1', 'family_tree', 'first active'),
  ('Active Diagram 2', 'correlation', 'second active');

INSERT INTO diagram
  (name, kind, description, deleted_at)
  VALUES
  ('Deleted Diagram', 'family_tree', 'should be filtered', now());
