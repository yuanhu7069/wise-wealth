-- B 期 · 埋点事件(ADR-B-005:入库不上报,自用期验链路)
--
-- 只增不减的事件表。列照 arch-v2 §2 落地,不记 user_id —— 自用期单用户,
-- 事件归属由「库里只有一个账号」隐含;多用户时加一列即可,不影响已有行。
--
-- payload 只存**枚举与步号**:金额永不入埋点(arch §8 日志脱敏)。
-- 这条约束由 `services/analytics_service.rs` 的 Event 枚举在类型层保证 ——
-- 每个变体自带的字段只能是页面 id / 步号 / 模式 id / 版本号,没有可以塞金额的位置。
CREATE TABLE analytics_events (
    id           UUID        PRIMARY KEY,
    event_type   TEXT        NOT NULL,
    payload_json JSONB       NOT NULL DEFAULT '{}'::jsonb,
    occurred_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 查验按「某类事件的时间序」与「某类事件按步号分组」进行(验收动作:一条 SQL 看事件分布
-- 与各步到达情况)。表只增不减,单列索引足够,不做 (type, payload) 的函数索引。
CREATE INDEX analytics_events_type_time_idx ON analytics_events (event_type, occurred_at);

COMMENT ON TABLE analytics_events IS '埋点事件(只入库不上报);payload 仅枚举与步号,金额永不入表';
COMMENT ON COLUMN analytics_events.event_type IS 'page_view / questionnaire_start / questionnaire_step_completed / questionnaire_completed / plan_generated';
