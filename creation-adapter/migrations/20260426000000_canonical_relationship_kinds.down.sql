DROP INDEX IF EXISTS uq_relationship_symmetric_active;
DROP INDEX IF EXISTS uq_relationship_directed_active;

ALTER TABLE relationship
    DROP CONSTRAINT IF EXISTS chk_relationship_no_active_self_relation;

ALTER TABLE relationship
    RENAME COLUMN kind TO canonical_kind;

CREATE TYPE relationship_kind_old AS ENUM (
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

ALTER TABLE relationship
    ADD COLUMN kind relationship_kind_old;

UPDATE relationship
SET kind = CASE canonical_kind::text
    WHEN 'adoptive_parent' THEN 'adopted_parent'::relationship_kind_old
    WHEN 'partner' THEN 'spouse'::relationship_kind_old
    ELSE canonical_kind::text::relationship_kind_old
END;

ALTER TABLE relationship
    ALTER COLUMN kind SET NOT NULL;

ALTER TABLE relationship
    DROP COLUMN canonical_kind;

DROP TYPE relationship_kind;

ALTER TYPE relationship_kind_old RENAME TO relationship_kind;

ALTER TABLE relationship
    DROP COLUMN IF EXISTS end_reason;
