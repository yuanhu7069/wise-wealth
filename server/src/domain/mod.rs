//! 领域层(纯结构体与纯函数,零 IO、零 HTTP 感知 —— ADR-B-004)。
//!
//! 本层是「一套内核」的落点:起步模式(免费)与将来的推演模式(Plus)共用同一套计算结果,
//! 只是暴露程度不同(产品 PRD §2.1)。

pub mod engine;
pub mod l2;
pub mod mode;
pub mod profile;

pub use engine::{
    BucketAmount, EmergencyStatus, EngineError, Notice, PlanResult, Trace, TraceUnit, solve,
};
pub use l2::{L2Allocation, L2Error};
pub use mode::{
    BucketSpec, Credibility, EmergencyFundRule, L2Class, L2Config, LibraryError, ModeConfig,
    ModeLibrary, RuleKind, ShareType,
};
pub use profile::{DrawdownResponse, Goal, Horizon, IncomeStability, Profile};
