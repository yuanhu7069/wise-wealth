/**
 * 埋点上报(RULE-019 / ADR-B-005)。**仅服务端** —— 事件在这一层触发,
 * 浏览器端不接触任何埋点入口(基线 §11.3:禁止散落客户端各处)。
 *
 * 只上报**页面才知道的两件事**:页面触达与问卷开始。其余三类(每步保存、
 * 问卷答全、方案生成)后端自己就知道,由 Rust 侧直接入库 —— 两个来源记同一件事,
 * 迟早会重复计数。
 *
 * 上报走 `after()`:响应发出之后才发请求。埋点失败不得影响主流程(基线 §11.3),
 * 而把 5 秒超时的 fetch await 在渲染路径上,等于让埋点决定页面什么时候出来 ——
 * after 把两件事彻底解耦,渲染不再持有这个请求。
 *
 * 与后端 `services/analytics_service.rs` 的 `Event::from_client` 白名单一一对应:
 * 事件名与 page_id 传错值后端会拒(422),但那是兜底;这里不该传错。
 */
import "server-only";
import { cookies } from "next/headers";
import { after } from "next/server";

import { apiPost } from "./api";
import { SESSION_COOKIE } from "./session";

/** 埋点只认这三张页面(design-v2 §1 页面表)。 */
export type PageId = "p01" | "p03" | "p04";

/**
 * 上报一条事件。**调用方拿不到结果**是刻意的:埋点是旁路观测,
 * 后端拒收或不可达都只留一行日志,不做任何分支 —— 页面对此无感知。
 */
async function track(event: string, pageId?: PageId): Promise<void> {
  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value;
  // 未登录不记:埋点属于登录后的产品流程,匿名触达无分析价值
  if (!token) return;

  // 令牌与请求体在此备好:after 的回调跑在响应之后,那时已不在请求作用域里
  const cookie = `${SESSION_COOKIE}=${token}`;
  const body = pageId ? { event, page_id: pageId } : { event };

  after(async () => {
    try {
      const { status, envelope } = await apiPost("/api/v1/analytics/events", body, cookie);
      if (status !== 200) {
        // 白名单拒收(422)是这里唯一「错在自己」的情况,不记下来就查不到
        console.warn(`[analytics] ${event} 被拒:${envelope.errorCode ?? `HTTP ${status}`}`);
      }
    } catch (e) {
      // 后端不可达:埋点不阻断主流程(基线 §11.3),但也不静默失败(基线 §10.2.4)
      console.warn(`[analytics] ${event} 上报失败:`, e instanceof Error ? e.message : e);
    }
  });
}

/** 页面触达(P01/P03/P04 服务端渲染时调用)。 */
export async function trackPageView(pageId: PageId): Promise<void> {
  await track("page_view", pageId);
}

/** 问卷开始(P03 首次进入且尚无草稿时调用;重复进入由后端去重)。 */
export async function trackQuestionnaireStart(): Promise<void> {
  await track("questionnaire_start");
}
