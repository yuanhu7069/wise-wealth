/**
 * 登录表单的状态类型与初值。
 *
 * 单独成文件:`"use server"` 文件只允许导出 async 函数,
 * 常量与类型必须放在普通模块里,否则 Next 会在运行时报
 * 「A "use server" file can only export async functions, found object」。
 */
export interface LoginState {
  /** 有值即为错误文案(空串表示无错误) */
  error: string;
}

export const initialLoginState: LoginState = { error: "" };
