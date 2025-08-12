-- Add up migration script here

CREATE TABLE IF NOT EXISTS `users` (
    `id` BINARY(16) NOT NULL PRIMARY KEY DEFAULT (UUID_TO_BIN(UUID(), 1)),
    `name` VARCHAR(100) NOT NULL,
    `email` VARCHAR(255) NOT NULL UNIQUE,
    `photo` VARCHAR(255) NOT NULL DEFAULT 'default.png',
    `password` VARCHAR(100) NOT NULL,
    `role` VARCHAR(50) NOT NULL DEFAULT 'user',
    -- `created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `created_at` DECIMAL(65, 6) NOT NULL DEFAULT (UNIX_TIMESTAMP(CURRENT_TIMESTAMP(6))),
    `tz_created_at` DATETIME(6) AS (FROM_UNIXTIME(created_at)) VIRTUAL NOT NULL,
    -- `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
    `updated_at` DECIMAL(65, 6) NOT NULL DEFAULT (UNIX_TIMESTAMP(CURRENT_TIMESTAMP(6))),
    `tz_updated_at` DATETIME(6) AS (FROM_UNIXTIME(updated_at)) VIRTUAL NOT NULL
);

CREATE INDEX users_email_idx ON users (email);


-- Diagramテーブル
CREATE TABLE IF NOT EXISTS diagram (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type ENUM('family_tree') NOT NULL,
    description TEXT,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    deleted_at DATETIME(6) NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Entityテーブル（diagram_id追加）
CREATE TABLE IF NOT EXISTS entity (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    diagram_id BIGINT NOT NULL,
    type ENUM('person') NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    deleted_at DATETIME(6) NULL,
    CONSTRAINT fk_entity_diagram FOREIGN KEY (diagram_id) REFERENCES diagram(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Personテーブル
CREATE TABLE IF NOT EXISTS person (
    entity_id BIGINT NOT NULL PRIMARY KEY,
    gender ENUM('male', 'female', 'other', 'unknown') DEFAULT 'unknown',
    birth_date DATE,
    death_date DATE,
    birthplace VARCHAR(255),
    residence VARCHAR(255),
    photo_url VARCHAR(512),
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    deleted_at DATETIME(6) NULL,
    CONSTRAINT fk_person_entity FOREIGN KEY (entity_id) REFERENCES entity(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Relationshipテーブル（diagram_id追加）
CREATE TABLE IF NOT EXISTS relationship (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    diagram_id BIGINT NOT NULL,
    source_entity_id BIGINT NOT NULL,
    target_entity_id BIGINT NOT NULL,
    type ENUM(
      'parent',
      'child',
      'spouse',
      'adopted_parent',
      'adopted_child',
      'divorced_spouse',
      'cohabitant',
      'step_parent',
      'step_child'
    ) NOT NULL,
    start_date DATE NULL,
    end_date DATE NULL,
    notes TEXT,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
    deleted_at DATETIME(6) NULL,
    CONSTRAINT fk_relationship_diagram FOREIGN KEY (diagram_id) REFERENCES diagram(id) ON DELETE CASCADE,
    CONSTRAINT fk_relationship_source_entity FOREIGN KEY (source_entity_id) REFERENCES entity(id) ON DELETE CASCADE,
    CONSTRAINT fk_relationship_target_entity FOREIGN KEY (target_entity_id) REFERENCES entity(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- インデックス
CREATE INDEX idx_entity_type ON entity(type);
CREATE INDEX idx_relationship_source_entity ON relationship(source_entity_id);
CREATE INDEX idx_relationship_target_entity ON relationship(target_entity_id);
CREATE INDEX idx_relationship_type ON relationship(type);
CREATE INDEX idx_entity_diagram ON entity(diagram_id);
CREATE INDEX idx_relationship_diagram ON relationship(diagram_id);