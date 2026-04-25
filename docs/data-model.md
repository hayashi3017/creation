# Data Model

Last updated: 2026-04-26

## Scope

This document summarizes the PostgreSQL schema and Rust-side model mapping used in this workspace.

Primary schema source:

- `creation-adapter/migrations/20240330073341_init.up.sql`
- `creation-adapter/migrations/20260411000000_explicit_primary_key_column_names.up.sql`

## Database enum types

Defined enum types:

- `diagram_kind`: `family_tree`, `correlation`
- `entity_kind`: `person`
- `gender_kind`: `male`, `female`, `other`, `unknown`
- `relationship_kind`:
  - `parent`
  - `adoptive_parent`
  - `step_parent`
  - `spouse`
  - `partner`
  - `cohabitant`

## Tables

### `users`

Purpose:

- Authentication and user identity.

Columns:

- `user_id UUID PRIMARY KEY DEFAULT uuid_generate_v4()`
- `name VARCHAR(100) NOT NULL`
- `email VARCHAR(255) NOT NULL UNIQUE`
- `photo VARCHAR(255) NOT NULL DEFAULT 'default.png'`
- `password VARCHAR(100) NOT NULL` (Argon2 hash is stored)
- `role VARCHAR(50) NOT NULL DEFAULT 'user'`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`

Indexes:

- `users_email_idx` on `email`

Rust mapping:

- DB row model: `creation-adapter/src/model/user.rs` (`UserTable`)
- API filtered model: `creation-service/src/model/user.rs` (`FilteredUser`)

### `diagram`

Purpose:

- Top-level diagram records.

Columns:

- `diagram_id BIGSERIAL PRIMARY KEY`
- `name VARCHAR(255) NOT NULL`
- `kind diagram_kind NOT NULL`
- `description TEXT`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `deleted_at TIMESTAMPTZ` (soft delete marker)

Rust mapping:

- DB row model: `creation-adapter/src/model/diagram.rs` (`DiagramTable`)
- API/domain model: `creation-service/src/model/diagram.rs` (`Diagram`)
- Enum mapping: `creation-service/src/model/diagram.rs` (`DiagramKind`)

### `entity`

Purpose:

- Entity nodes belonging to a diagram.

Columns:

- `entity_id BIGSERIAL PRIMARY KEY`
- `diagram_id BIGINT NOT NULL REFERENCES diagram(diagram_id) ON DELETE CASCADE`
- `kind entity_kind NOT NULL`
- `name VARCHAR(255) NOT NULL`
- `description TEXT`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `deleted_at TIMESTAMPTZ`

Indexes:

- `idx_entity_kind` on `kind`
- `idx_entity_diagram` on `diagram_id`

### `person`

Purpose:

- Person-specific details for `entity(kind='person')`.

Columns:

- `entity_id BIGINT PRIMARY KEY REFERENCES entity(entity_id) ON DELETE CASCADE`
- `gender gender_kind DEFAULT 'unknown'`
- `birth_date DATE`
- `death_date DATE`
- `birthplace VARCHAR(255)`
- `residence VARCHAR(255)`
- `photo_url VARCHAR(512)`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `deleted_at TIMESTAMPTZ`

Rust mapping:

- DB row model: `creation-adapter/src/model/person.rs` (`PersonTable`)
- API/domain model: `creation-service/src/model/person.rs` (`Person`)
- Enum mapping: `creation-service/src/model/person.rs` (`GenderKind`)

### `relationship`

Purpose:

- Links between entities inside one diagram.

Columns:

- `relationship_id BIGSERIAL PRIMARY KEY`
- `diagram_id BIGINT NOT NULL REFERENCES diagram(diagram_id) ON DELETE CASCADE`
- `source_entity_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE`
- `target_entity_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE`
- `kind relationship_kind NOT NULL`
- `start_date DATE`
- `end_date DATE`
- `end_reason VARCHAR(32)`
- `notes TEXT`
- `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`
- `deleted_at TIMESTAMPTZ`

Indexes:

- `idx_relationship_source_entity` on `source_entity_id`
- `idx_relationship_target_entity` on `target_entity_id`
- `idx_relationship_kind` on `kind`
- `idx_relationship_diagram` on `diagram_id`

### `tree_path`

Purpose:

- Closure table for ancestor/descendant traversal.

Columns:

- `ancestor_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE`
- `descendant_id BIGINT NOT NULL REFERENCES entity(entity_id) ON DELETE CASCADE`
- `depth INT NOT NULL`
- `PRIMARY KEY (ancestor_id, descendant_id)`

Indexes:

- `idx_tree_path_ancestor` on `ancestor_id`
- `idx_tree_path_descendant` on `descendant_id`

## Current implementation coverage

Actively used by repository code:

- `users`
- `diagram`
- `entity`
- `person`

Defined in migration but not yet wired in current repositories/handlers:
- none

## Data handling notes

- User email is normalized to lowercase at registration and login query paths.
- Passwords are hashed with Argon2 before insert (`creation-adapter/src/repository/user.rs`).
- JWT `sub` claim stores user UUID as string.
- `relationship` write APIs accept canonical stored relationship kinds only.
- `tree_path` is rebuilt from active `parent` and `adoptive_parent` lineage edges only.
