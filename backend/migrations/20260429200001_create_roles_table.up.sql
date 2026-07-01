CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO roles (name, description) VALUES
    ('guest',       '訪客（未登入）'),
    ('member',      '一般會員'),
    ('admin',       '後台全功能'),
    ('super_admin', '含管理帳號/角色');
