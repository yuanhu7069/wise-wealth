-- B 期 · 问卷档案(profiles)
--
-- 一行/用户,upsert 更新。同时承载「草稿」与「已完成」两种状态:
--   draft_step 记录下一步该答哪一步(1-6);questionnaire_completed 在步 5 保存后置真。
-- 分步保存而非一次性提交:用户中途关页面再回来要能接着答(RULE-003)。
--
-- 金额一律 BIGINT 分、字段名带 _cents;时间 timestamptz(UTC)。
-- 社保/商保/房贷余额本期仅入库备查(产品 PRD §13.3 把房贷对比排在后续期)。
CREATE TABLE profiles (
    id                        UUID        PRIMARY KEY,
    user_id                   UUID        NOT NULL REFERENCES users(id),
    -- 问卷答案
    horizon                   TEXT,
    drawdown_response         TEXT,
    income_stability          TEXT,
    has_social_security       BOOLEAN     NOT NULL DEFAULT FALSE,
    has_commercial_insurance  BOOLEAN     NOT NULL DEFAULT FALSE,
    mortgage_balance_cents    BIGINT,     -- 可空:选填,本期只记录
    dependents                SMALLINT,
    -- 财务数字(分)
    inflow_cents              BIGINT,
    expense_fixed_monthly_cents BIGINT,
    savings_cents             BIGINT,
    goal                      TEXT,
    -- 进度
    draft_step                SMALLINT    NOT NULL DEFAULT 1,
    questionnaire_completed   BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 一个用户一份档案(不软删:档案是 upsert 的单行,B 期无删除入口)
CREATE UNIQUE INDEX profiles_user_id_key ON profiles (user_id);

COMMENT ON COLUMN profiles.draft_step IS '下一步该答的步号 1-6;6 表示已到推荐步';
COMMENT ON COLUMN profiles.mortgage_balance_cents IS '本期仅记录,不参与引擎计算';
