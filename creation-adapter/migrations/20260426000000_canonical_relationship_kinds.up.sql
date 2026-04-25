ALTER TABLE relationship
    ADD COLUMN IF NOT EXISTS end_reason VARCHAR(32);

DROP INDEX IF EXISTS uq_relationship_symmetric_active;
DROP INDEX IF EXISTS uq_relationship_directed_active;

ALTER TABLE relationship
    DROP CONSTRAINT IF EXISTS chk_relationship_no_active_self_relation;

ALTER TABLE relationship
    RENAME COLUMN kind TO old_kind;

ALTER TYPE relationship_kind RENAME TO relationship_kind_old;

CREATE TYPE relationship_kind AS ENUM (
    'parent',
    'adoptive_parent',
    'step_parent',
    'spouse',
    'partner',
    'cohabitant'
);

ALTER TABLE relationship
    ADD COLUMN kind relationship_kind;

UPDATE relationship
SET
    source_entity_id = target_entity_id,
    target_entity_id = source_entity_id,
    kind = 'parent'
WHERE old_kind::text = 'child';

UPDATE relationship
SET
    kind = 'parent'
WHERE old_kind::text = 'parent';

UPDATE relationship
SET
    kind = 'adoptive_parent'
WHERE old_kind::text IN ('adopted_parent', 'adoptive_parent');

UPDATE relationship
SET
    source_entity_id = target_entity_id,
    target_entity_id = source_entity_id,
    kind = 'adoptive_parent'
WHERE old_kind::text = 'adopted_child';

UPDATE relationship
SET
    kind = 'step_parent'
WHERE old_kind::text = 'step_parent';

UPDATE relationship
SET
    source_entity_id = target_entity_id,
    target_entity_id = source_entity_id,
    kind = 'step_parent'
WHERE old_kind::text = 'step_child';

UPDATE relationship
SET
    kind = 'spouse'
WHERE old_kind::text = 'spouse';

UPDATE relationship
SET
    kind = 'partner'
WHERE old_kind::text = 'partner';

UPDATE relationship
SET
    kind = 'spouse',
    end_reason = COALESCE(end_reason, 'divorce')
WHERE old_kind::text = 'divorced_spouse';

UPDATE relationship
SET
    kind = 'cohabitant'
WHERE old_kind::text = 'cohabitant';

-- `sibling` is derived-only after RFC 0013. Preserve the row as soft-deleted
-- historical data while assigning a castable canonical kind.
UPDATE relationship
SET
    kind = 'parent',
    deleted_at = COALESCE(deleted_at, now()),
    updated_at = now()
WHERE old_kind::text = 'sibling';

UPDATE relationship
SET
    source_entity_id = target_entity_id,
    target_entity_id = source_entity_id
WHERE
    kind IN ('spouse', 'partner', 'cohabitant')
    AND source_entity_id > target_entity_id;

ALTER TABLE relationship
    ALTER COLUMN kind SET NOT NULL;

ALTER TABLE relationship
    DROP COLUMN old_kind;

DROP TYPE relationship_kind_old;

UPDATE relationship
SET
    deleted_at = COALESCE(deleted_at, now()),
    updated_at = now()
WHERE source_entity_id = target_entity_id;

WITH duplicates AS (
    SELECT
        relationship_id,
        row_number() OVER (
            PARTITION BY diagram_id, source_entity_id, target_entity_id, kind
            ORDER BY relationship_id
        ) AS rn
    FROM relationship
    WHERE
        deleted_at IS NULL
        AND kind IN ('parent', 'adoptive_parent', 'step_parent')
)
UPDATE relationship AS r
SET
    deleted_at = now(),
    updated_at = now()
FROM duplicates
WHERE
    r.relationship_id = duplicates.relationship_id
    AND duplicates.rn > 1;

WITH duplicates AS (
    SELECT
        relationship_id,
        row_number() OVER (
            PARTITION BY diagram_id, source_entity_id, target_entity_id, kind
            ORDER BY relationship_id
        ) AS rn
    FROM relationship
    WHERE
        deleted_at IS NULL
        AND kind IN ('spouse', 'partner', 'cohabitant')
)
UPDATE relationship AS r
SET
    deleted_at = now(),
    updated_at = now()
FROM duplicates
WHERE
    r.relationship_id = duplicates.relationship_id
    AND duplicates.rn > 1;

ALTER TABLE relationship
    ADD CONSTRAINT chk_relationship_no_active_self_relation
    CHECK (deleted_at IS NOT NULL OR source_entity_id <> target_entity_id);

CREATE UNIQUE INDEX uq_relationship_directed_active
ON relationship (
    diagram_id,
    source_entity_id,
    target_entity_id,
    kind
)
WHERE deleted_at IS NULL
    AND kind IN ('parent', 'adoptive_parent', 'step_parent');

CREATE UNIQUE INDEX uq_relationship_symmetric_active
ON relationship (
    diagram_id,
    LEAST(source_entity_id, target_entity_id),
    GREATEST(source_entity_id, target_entity_id),
    kind
)
WHERE deleted_at IS NULL
    AND kind IN ('spouse', 'partner', 'cohabitant');

TRUNCATE TABLE tree_path;

WITH RECURSIVE
active_entities AS (
    SELECT
        e.entity_id
    FROM entity AS e
    INNER JOIN diagram AS d
        ON d.diagram_id = e.diagram_id
        AND d.deleted_at IS NULL
    WHERE e.deleted_at IS NULL
),
lineage_edges AS (
    SELECT
        r.source_entity_id AS ancestor_id,
        r.target_entity_id AS descendant_id
    FROM relationship AS r
    INNER JOIN diagram AS d
        ON d.diagram_id = r.diagram_id
        AND d.deleted_at IS NULL
    INNER JOIN entity AS source
        ON source.entity_id = r.source_entity_id
        AND source.diagram_id = r.diagram_id
        AND source.deleted_at IS NULL
    INNER JOIN entity AS target
        ON target.entity_id = r.target_entity_id
        AND target.diagram_id = r.diagram_id
        AND target.deleted_at IS NULL
    WHERE
        r.deleted_at IS NULL
        AND r.kind IN ('parent', 'adoptive_parent')
),
paths AS (
    SELECT
        entity_id AS ancestor_id,
        entity_id AS descendant_id,
        0 AS depth,
        ARRAY[entity_id] AS visited_entity_ids
    FROM active_entities
    UNION ALL
    SELECT
        paths.ancestor_id,
        lineage_edges.descendant_id,
        paths.depth + 1,
        paths.visited_entity_ids || lineage_edges.descendant_id
    FROM paths
    INNER JOIN lineage_edges
        ON lineage_edges.ancestor_id = paths.descendant_id
    WHERE NOT lineage_edges.descendant_id = ANY(paths.visited_entity_ids)
),
deduped_paths AS (
    SELECT
        ancestor_id,
        descendant_id,
        MIN(depth) AS depth
    FROM paths
    GROUP BY ancestor_id, descendant_id
)
INSERT INTO tree_path (ancestor_id, descendant_id, depth)
SELECT ancestor_id, descendant_id, depth
FROM deduped_paths;
