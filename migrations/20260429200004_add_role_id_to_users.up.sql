CREATE TABLE user_roles (
    user_id BIGINT  NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

-- 現有 user 預設指派 member role
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id FROM users u, roles r WHERE r.name = 'member';
