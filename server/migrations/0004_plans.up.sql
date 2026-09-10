-- B 期 · 方案(plans + plan_buckets)
--
-- 方案一经生成即冻结:档案快照、L2 配置、应急金状态、提示与推理链全部存快照 ——
-- 模式库配置日后修改,不影响历史方案的呈现(历史就该是当时算出来的样子)。
--
-- 版本化:每次生成插入新行而非覆盖,旧版本保留(产品 PRD 追踪期要对比历史快照)。
-- 「单一 active」由**部分唯一索引**在数据库层保证,不依赖应用层自觉。
CREATE TABLE plans (
    id                    UUID        PRIMARY KEY,
    user_id               UUID        NOT NULL REFERENCES users(id),
    l1_mode               TEXT        NOT NULL,
    l2_mode               TEXT        NOT NULL,
    version               INTEGER     NOT NULL,
    is_active             BOOLEAN     NOT NULL DEFAULT TRUE,
    -- 快照:生成当时的档案(数字变了也能回溯"当时是按什么算的")
    profile_snapshot_json JSONB       NOT NULL,
    -- 快照:L2 大类配置(含匹配理由)
    l2_allocation_json    JSONB       NOT NULL,
    -- 快照:应急金状态与提示(达标/缺口/短久期),API 直接回读,不重算
    emergency_json        JSONB       NOT NULL,
    notices_json          JSONB       NOT NULL DEFAULT '[]'::jsonb,
    -- 快照:推理链(B 期不展示,为 Plus 预埋)
    traces_json           JSONB       NOT NULL DEFAULT '[]'::jsonb,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 版本号在用户内唯一
CREATE UNIQUE INDEX plans_user_version_key ON plans (user_id, version);
-- 任一用户至多一个 active 方案(ADR-B-003:约束落在数据库层)
CREATE UNIQUE INDEX plans_one_active_per_user ON plans (user_id) WHERE is_active;

CREATE TABLE plan_buckets (
    id                   UUID     PRIMARY KEY,
    plan_id              UUID     NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    bucket_id            TEXT     NOT NULL,
    name                 TEXT     NOT NULL,
    purpose              TEXT     NOT NULL DEFAULT '',
    amount_monthly_cents BIGINT   NOT NULL,
    target_cents         BIGINT,
    sort_order           SMALLINT NOT NULL DEFAULT 0
);

CREATE INDEX plan_buckets_plan_id_idx ON plan_buckets (plan_id);

COMMENT ON INDEX plans_one_active_per_user IS '单一 active 由数据库保证,不依赖应用层';
COMMENT ON COLUMN plans.profile_snapshot_json IS '生成当时的档案快照,用于回溯';
