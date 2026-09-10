-- B 期 · 单用户账号(ADR-B-002)
--
-- 无注册页:账号由 `wise-wealth-server seed-user` 从环境变量幂等写入。
-- tier 为产品分层预留(free/plus),B 期恒为 free。
CREATE TABLE users (
    id            UUID        PRIMARY KEY,
    username      TEXT        NOT NULL UNIQUE,
    password_hash TEXT        NOT NULL,
    tier          TEXT        NOT NULL DEFAULT 'free',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- password_hash:argon2 哈希,永不返回 API、永不进日志(基线 §8.3)
COMMENT ON COLUMN users.password_hash IS 'argon2 哈希;禁止明文,禁止进日志';
COMMENT ON COLUMN users.tier IS '产品分层预留,DEFAULT 0 期恒为 free';
