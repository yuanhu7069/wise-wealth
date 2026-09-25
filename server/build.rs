//! 构建期扫描模式库与知识库目录,生成内嵌配置清单。
//!
//! 存在的理由只有一个:让 ADR-B-001 的「**加模式 = 加文件,不改代码**」与 ADR-G-001 的
//! 「**加知识内容 = 加文件,不改代码**」名副其实。若把清单写成模块里的字面量数组,
//! 新增条目仍要回去补一行 `include_str!` —— 承诺就只是说法,不是事实。
//! 这里把它变成事实:目录里放一份 TOML,重新构建即生效。

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let modes_dir = PathBuf::from("config/modes");
    let l2_dir = modes_dir.join("l2");
    let knowledge_dir = PathBuf::from("config/knowledge");
    println!("cargo:rerun-if-changed=config/modes");
    println!("cargo:rerun-if-changed=config/modes/l2");
    println!("cargo:rerun-if-changed=config/knowledge");

    let l1 = collect(&modes_dir, "模式库");
    let l2 = collect(&l2_dir, "L2 配置目录");
    let knowledge = collect(&knowledge_dir, "知识库");

    let mut out = String::new();
    out.push_str("// 本文件由 build.rs 生成,请勿手改。新增模式 = 在 config/modes/ 放一份 TOML。\n\n");
    emit(&mut out, "EMBEDDED_MODES", &l1, "L1 模式配置");
    out.push('\n');
    emit(&mut out, "EMBEDDED_L2", &l2, "L2 大类配置");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR 由 cargo 注入");
    let dest = Path::new(&out_dir).join("mode_sources.rs");
    std::fs::write(&dest, out).unwrap_or_else(|e| panic!("生成 {dest:?} 失败: {e}"));

    // 知识库单独一个生成文件:两个模块各自 include 自己用到的常量,
    // 避免共享文件把「对方用不到的常量」变成 dead_code 警告。
    let mut kout = String::new();
    kout.push_str("// 本文件由 build.rs 生成,请勿手改。新增知识内容 = 在 config/knowledge/ 放一份 TOML。\n\n");
    emit(&mut kout, "EMBEDDED_KNOWLEDGE", &knowledge, "知识库文章(G 期 ADR-G-001)");
    let kdest = Path::new(&out_dir).join("knowledge_sources.rs");
    std::fs::write(&kdest, kout).unwrap_or_else(|e| panic!("生成 {kdest:?} 失败: {e}"));
}

/// 生成一个 `&[(id, 配置文本)]` 常量。路径取绝对路径 —— `include_str!` 在生成文件里
/// 展开,相对路径会以 OUT_DIR 为基准解析。
fn emit(out: &mut String, name: &str, files: &[(String, PathBuf)], doc: &str) {
    let _ = writeln!(out, "/// {doc}(构建期扫描生成)。");
    let _ = writeln!(out, "pub(crate) const {name}: &[(&str, &str)] = &[");
    for (id, path) in files {
        let abs = std::fs::canonicalize(path)
            .unwrap_or_else(|e| panic!("无法解析 {path:?} 的绝对路径: {e}"));
        let _ = writeln!(out, "    ({id:?}, include_str!({abs:?})),");
    }
    out.push_str("];\n");
}

/// 扫描目录下的全部 `.toml`,按文件名排序返回 `(文件名词干, 路径)`。
///
/// 目录为空即构建失败:模式库与知识库都是编译期资产,空库不是「降级」而是配置事故。
fn collect(dir: &Path, label: &str) -> Vec<(String, PathBuf)> {
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{label}目录不可读 {}: {e}", dir.display()));

    let mut found: Vec<(String, PathBuf)> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
        .map(|p| {
            let stem = p
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            (stem, p)
        })
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));

    if found.is_empty() {
        panic!("{label}目录为空:{}", dir.display());
    }
    found
}
