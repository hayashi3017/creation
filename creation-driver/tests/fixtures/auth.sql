CREATE TABLE IF NOT EXISTS users (
    user_id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    photo VARCHAR(255) NOT NULL DEFAULT 'default.png',
    password VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO users
  (user_id, email, name, password, photo, role)
  VALUES
  (
    '00000000-0000-0000-0000-000000000010',
    'login@example.com',
    'login_user',
    '$argon2id$v=19$m=102400,t=2,p=8$eYYQoINIU5/Q6q5pk51pCA$IS1hHgNNyhLyp0JWCJjN3w',
    'default.png',
    'user'
  ),
  (
    '00000000-0000-0000-0000-000000000011',
    'badpass@example.com',
    'badpass_user',
    '$argon2id$v=19$m=102400,t=2,p=8$eYYQoINIU5/Q6q5pk51pCA$IS1hHgNNyhLyp0JWCJjN3w',
    'default.png',
    'user'
  ),
  (
    '00000000-0000-0000-0000-000000000020',
    'me@example.com',
    'me_user',
    '$argon2id$v=19$m=102400,t=2,p=8$eYYQoINIU5/Q6q5pk51pCA$IS1hHgNNyhLyp0JWCJjN3w',
    'default.png',
    'user'
  ),
  (
    '00000000-0000-0000-0000-000000000021',
    'logout@example.com',
    'logout_user',
    '$argon2id$v=19$m=102400,t=2,p=8$eYYQoINIU5/Q6q5pk51pCA$IS1hHgNNyhLyp0JWCJjN3w',
    'default.png',
    'user'
  );
