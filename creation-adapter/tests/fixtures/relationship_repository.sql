DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'diagram_kind') THEN
        CREATE TYPE diagram_kind AS ENUM ('family_tree', 'correlation');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'entity_kind') THEN
        CREATE TYPE entity_kind AS ENUM ('person');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'relationship_kind') THEN
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
    END IF;
END
$$;

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

INSERT INTO diagram
  (diagram_id, name, kind, description, deleted_at)
VALUES
  (1, 'Relationship Diagram 1', 'family_tree', 'first diagram', NULL),
  (2, 'Relationship Diagram 2', 'family_tree', 'second diagram', NULL),
  (3, 'Deleted Relationship Diagram', 'family_tree', 'soft deleted diagram', now());

SELECT setval(pg_get_serial_sequence('diagram', 'diagram_id'), 3, true);

INSERT INTO entity
  (entity_id, diagram_id, kind, name, description, deleted_at)
VALUES
  (1, 1, 'person', 'Ancestor', NULL, NULL),
  (2, 1, 'person', 'Parent', NULL, NULL),
  (3, 1, 'person', 'Child', NULL, NULL),
  (4, 2, 'person', 'Other Diagram Parent', NULL, NULL),
  (5, 2, 'person', 'Other Diagram Child', NULL, NULL),
  (6, 1, 'person', 'Deleted Entity', NULL, now()),
  (7, 1, 'person', 'Extra Child', NULL, NULL),
  (8, 3, 'person', 'Deleted Diagram Parent', NULL, NULL),
  (9, 3, 'person', 'Deleted Diagram Child', NULL, NULL);

SELECT setval(pg_get_serial_sequence('entity', 'entity_id'), 9, true);

INSERT INTO relationship
  (relationship_id, diagram_id, source_entity_id, target_entity_id, kind, notes, deleted_at)
VALUES
  (1, 1, 1, 2, 'parent', 'ancestor to parent', NULL),
  (2, 1, 2, 3, 'parent', 'parent to child', NULL),
  (3, 2, 4, 5, 'parent', 'other diagram', NULL),
  (4, 1, 1, 7, 'parent', 'deleted edge', now()),
  (5, 3, 8, 9, 'parent', 'soft deleted diagram edge', NULL);

SELECT setval(pg_get_serial_sequence('relationship', 'relationship_id'), 5, true);
