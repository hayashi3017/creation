DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'diagram_kind') THEN
        CREATE TYPE diagram_kind AS ENUM ('family_tree', 'correlation');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'entity_kind') THEN
        CREATE TYPE entity_kind AS ENUM ('person');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'gender_kind') THEN
        CREATE TYPE gender_kind AS ENUM ('male', 'female', 'other', 'unknown');
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

CREATE TABLE IF NOT EXISTS person (
    entity_id BIGINT PRIMARY KEY REFERENCES entity(entity_id) ON DELETE CASCADE,
    first_name VARCHAR(255),
    middle_name VARCHAR(255),
    last_name VARCHAR(255),
    first_name_kana VARCHAR(255),
    middle_name_kana VARCHAR(255),
    last_name_kana VARCHAR(255),
    first_name_romaji VARCHAR(255),
    middle_name_romaji VARCHAR(255),
    last_name_romaji VARCHAR(255),
    gender gender_kind DEFAULT 'unknown',
    birth_date DATE,
    death_date DATE,
    birthplace VARCHAR(255),
    deathplace VARCHAR(255),
    residence VARCHAR(255),
    photo_url VARCHAR(512),
    profile_text TEXT,
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
  (diagram_id, world_id, name, kind, description)
  VALUES
  (1, 1, 'Repository Diagram 1', 'family_tree', 'first diagram'),
  (2, 1, 'Repository Diagram 2', 'correlation', 'second diagram');

SELECT setval(pg_get_serial_sequence('diagram', 'diagram_id'), 2, true);

INSERT INTO entity
  (entity_id, world_id, kind, name, description, deleted_at)
  VALUES
  (1, 1, 'person', 'Active Person 1', 'first active', NULL),
  (2, 1, 'person', 'Active Person 2', NULL, NULL),
  (3, 1, 'person', 'Other Diagram Person', 'other diagram', NULL),
  (4, 1, 'person', 'Deleted Entity Person', 'deleted entity', now()),
  (5, 1, 'person', 'Deleted Person Row', 'deleted person', NULL);

SELECT setval(pg_get_serial_sequence('entity', 'entity_id'), 5, true);

INSERT INTO diagram_entity
  (diagram_id, entity_id)
  VALUES
  (1, 1),
  (1, 2),
  (2, 3),
  (1, 4),
  (1, 5);

INSERT INTO person
  (entity_id, gender, birth_date, death_date, birthplace, residence, photo_url, deleted_at)
  VALUES
  (1, 'male', '1990-01-01', NULL, 'Tokyo', 'Osaka', 'https://example.com/1.png', NULL),
  (2, 'female', NULL, NULL, 'Kyoto', NULL, NULL, NULL),
  (3, 'other', NULL, NULL, 'Nagoya', NULL, NULL, NULL),
  (4, 'unknown', NULL, NULL, 'Sapporo', NULL, NULL, NULL),
  (5, 'female', NULL, NULL, 'Fukuoka', NULL, NULL, now());
