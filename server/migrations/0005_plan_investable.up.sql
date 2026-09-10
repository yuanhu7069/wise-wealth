-- B 期 · 首页摘要(ticket 06):「每月可投资」冻结列
--
-- 为什么存成列而不是读时从 plan_buckets 里挑:哪一个桶是投资桶由**生成当时**的模式配置
-- 决定(`is_investable`),配置日后改了桶 id 不该改写历史方案的首页摘要 ——
-- 与方案其余部分同一原则:生成即快照。
--
-- DEFAULT 0 只为让 0005 之前已生成的方案能直接升级(那几行首页摘要显示 ¥0.00);
-- 新方案一律由引擎输出显式写入。
ALTER TABLE plans
    ADD COLUMN investable_monthly_cents BIGINT NOT NULL DEFAULT 0;

COMMENT ON COLUMN plans.investable_monthly_cents IS '生成当时投资桶的每月转入(分),首页摘要的「每月可投资」';
