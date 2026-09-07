/**
 * errorCode → 用户文案映射(RULE-008 三要素:发生了什么 + 为什么 + 怎么办)。
 * 机器可读的 errorCode 只出后端;文案唯一来源是本表,禁止在组件里内联错误文案。
 * 参照基线 §6.2 错误码表与 prd-a.md §8.4。
 */

export interface ErrorCopy {
  /** 标题:发生了什么 */
  title: string;
  /** 说明:为什么 + 怎么办 */
  description: string;
}

/** ERR-001(后端不可达)与 ERR-002(库不可达)文案来自 prd-a.md §8.4。 */
export const ERROR_COPY: Record<string, ErrorCopy> = {
  // ERR-001:后端进程不可达(fetch 层失败)
  ERR_001: {
    title: "后端服务未响应",
    description: "后端服务可能没有启动。请运行 scripts/start.sh 启动后再试,或点击重试。",
  },
  // ERR-002:数据库不可达(health 信封 db=error)
  ERR_002: {
    title: "数据库连接失败",
    description: "服务在线,但数据库暂时不可达。请检查数据库实例状态,恢复后点击重试。",
  },
  // 通用兜底(基线 §6.2)
  VALIDATION_ERROR: {
    title: "提交的内容有误",
    description: "请检查填写的内容后重试。",
  },
  UNAUTHORIZED: {
    title: "登录已过期",
    description: "请重新登录后再操作。",
  },
  FORBIDDEN: {
    title: "没有权限",
    description: "当前身份无权执行这个操作。",
  },
  NOT_FOUND: {
    title: "没有找到这条内容",
    description: "它可能已被删除,请刷新后再试。",
  },
  CONFLICT: {
    title: "数据已变更",
    description: "请刷新页面后重试。",
  },
  RATE_LIMITED: {
    title: "操作太频繁了",
    description: "请稍后再试。",
  },
  INTERNAL_ERROR: {
    title: "服务出了点问题",
    description: "已记录该问题,请稍后重试。",
  },
};

/** 兜底文案:未知 errorCode 时使用(禁止把技术细节透给用户)。 */
export function errorCopyOf(code: string | null | undefined): ErrorCopy {
  if (code && ERROR_COPY[code]) return ERROR_COPY[code];
  return ERROR_COPY.INTERNAL_ERROR;
}
