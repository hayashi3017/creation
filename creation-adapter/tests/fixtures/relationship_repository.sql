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
            'adoptive_parent',
            'step_parent',
            'spouse',
            'partner',
            'cohabitant'
        );
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

CREATE TABLE IF NOT EXISTS relationship (
    relationship_id BIGSERIAL PRIMARY KEY,
    world_id BIGINT NOT NULL REFERENCES world(world_id) ON DELETE CASCADE,
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
    world_id BIGINT NOT NULL REFERENCES world(world_id) ON DELETE CASCADE,
    ancestor_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    descendant_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE,
    depth INT NOT NULL,
    PRIMARY KEY (world_id, ancestor_id, descendant_id)
);

INSERT INTO world
  (world_id, name, description)
VALUES
  (1, 'Relationship Repository World', 'fixture world');

SELECT setval(pg_get_serial_sequence('world', 'world_id'), 1, true);

INSERT INTO diagram
  (diagram_id, world_id, name, kind, description, deleted_at)
VALUES
  (1, 1, 'Relationship Diagram 1', 'family_tree', 'first diagram', NULL),
  (2, 1, 'Relationship Diagram 2', 'family_tree', 'second diagram', NULL),
  (3, 1, 'Deleted Relationship Diagram', 'family_tree', 'soft deleted diagram', now());

SELECT setval(pg_get_serial_sequence('diagram', 'diagram_id'), 3, true);

INSERT INTO entity
  (entity_id, world_id, kind, name, description, deleted_at)
VALUES
  (1, 1, 'person', 'Ancestor', NULL, NULL),
  (2, 1, 'person', 'Parent', NULL, NULL),
  (3, 1, 'person', 'Child', NULL, NULL),
  (4, 1, 'person', 'Other Diagram Parent', NULL, NULL),
  (5, 1, 'person', 'Other Diagram Child', NULL, NULL),
  (6, 1, 'person', 'Deleted Entity', NULL, now()),
  (7, 1, 'person', 'Extra Child', NULL, NULL),
  (8, 1, 'person', 'Deleted Diagram Parent', NULL, NULL),
  (9, 1, 'person', 'Deleted Diagram Child', NULL, NULL);

SELECT setval(pg_get_serial_sequence('entity', 'entity_id'), 9, true);

INSERT INTO diagram_entity
  (diagram_id, entity_id)
VALUES
  (1, 1),
  (1, 2),
  (1, 3),
  (2, 4),
  (2, 5),
  (1, 6),
  (1, 7),
  (3, 8),
  (3, 9);

INSERT INTO relationship
  (relationship_id, world_id, source_entity_id, target_entity_id, kind, notes, deleted_at)
VALUES
  (1, 1, 1, 2, 'parent', 'ancestor to parent', NULL),
  (2, 1, 2, 3, 'parent', 'parent to child', NULL),
  (3, 1, 4, 5, 'parent', 'other diagram', NULL),
  (4, 1, 1, 7, 'parent', 'deleted edge', now()),
  (5, 1, 8, 9, 'parent', 'soft deleted diagram edge', NULL);

SELECT setval(pg_get_serial_sequence('relationship', 'relationship_id'), 5, true);
