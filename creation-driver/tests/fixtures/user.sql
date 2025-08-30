CREATE TABLE IF NOT EXISTS users (
    id BINARY(16) NOT NULL PRIMARY KEY DEFAULT (UUID_TO_BIN(UUID(), 1)),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    photo VARCHAR(255) NOT NULL DEFAULT 'default.png',
    password VARCHAR(100) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX users_email_idx ON users (email);

INSERT INTO users
  (email, name, password, photo, role)
  VALUES
  ('test1@example.com', 'test1_user', 'test1_password', 'test1.png', 'user'),
  ('test2@example.com', 'test2_user', 'test2_password', 'test2.png', 'user'),
  ('test3@example.com', 'test3_user', 'test3_password', 'test3.png', 'user');
