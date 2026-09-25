"use server";

/**
 * P06 模式库生成入口的说明(server 侧无独立动作)。
 *
 * P06 的「用此模式生成方案」直接复用 plan/actions.ts 的 generatePlanAction
 * (RULE-034:生成语义与问卷路径完全一致,同一端点同一校验),
 * 仅以 entry="mode_lib" 区分埋点口径(prd-f §9.5)。
 * 本文件只是把这一决策显式化,避免后人另起炉灶写第二份生成动作。
 */
export { generatePlanAction } from "@/app/plan/actions";
