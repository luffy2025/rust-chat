-- Add migration script here
-- worksapce for users
CREATE TABLE IF NOT EXISTS workspaces (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    owner_id BIGINT NOT NULL REFERENCES users(id),
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);
-- alter user table to add ws_id
ALTER TABLE users
ADD COLUMN ws_id BIGINT REFERENCES workspaces(id);
-- alter chat table to add ws_id
ALTER TABLE chats
ADD COLUMN ws_id BIGINT REFERENCES workspaces(id);
-- insert default user and workspace
BEGIN;
INSERT INTO users (id, fullname, email, password_hash)
VALUES (0, 'super user', 'super@none.org', '');
INSERT INTO workspaces (id, name, owner_id)
VALUES (0, 'none', 0);
UPDATE users
SET ws_id = 0
WHERE id = 0;
COMMIT;
-- alter user table to make ws_id not null
ALTER TABLE users
ALTER COLUMN ws_id
SET NOT NULL;
