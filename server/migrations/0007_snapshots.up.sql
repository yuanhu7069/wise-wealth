-- E 期 · 月度快照(追踪模块,ADR-E-001)
--
-- 「每月一次快照,不是逐笔记账」是产品最关键的结构决策(产品 PRD §4.4.1):
-- 每用户每自然月至多一条,由 UNIQUE 索引在数据库层兜底(同 ADR-B-003 的立场)。
-- 同月再提交 = 覆盖(UPDATE),应用层不做第二个真源。
--
-- balances 用 JSONB(键 = 桶 id,值 = 分):桶集合随方案版本变化(3 桶或 4 桶,
-- 切换 L1 模式后完全不同),关系列无法表达动态桶集;键集合的合法性由 service
-- 对照提交时 active 方案校验兜底(多桶/少桶/未知桶一律 422)。
--
-- plan_id 冻结提交时的方案版本(RULE-028:历史快照不可变)。
-- 方案版本本身永存(ADR-B-003:无级联删除),故不设 ON DELETE。
CREATE TABLE snapshots (
    id            UUID        PRIMARY KEY,
    user_id       UUID        NOT NULL REFERENCES users(id),
    plan_id       UUID        NOT NULL REFERENCES plans(id),
    month         DATE        NOT NULL,
    balances      JSONB       NOT NULL,
    special_month BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 每用户每自然月至多一条(覆盖语义的数据库层兜底,ADR-E-001)
CREATE UNIQUE INDEX snapshots_user_month_key ON snapshots (user_id, month);
-- 历史列表与「最新一条」查询按月倒序
CREATE INDEX snapshots_user_month_idx ON snapshots (user_id, month DESC);

COMMENT ON TABLE snapshots IS '月度快照:每用户每自然月至多一条;balances 键=桶 id,值=分;金额不入日志与埋点';
COMMENT ON COLUMN snapshots.plan_id IS '提交时的方案版本(冻结,RULE-028);方案版本无级联删除';
COMMENT ON COLUMN snapshots.special_month IS '本月特殊(RULE-024):不触发偏离提示,也不作为后续环比基准';
