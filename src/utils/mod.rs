pub mod crypto;
pub mod jwt;
pub mod serde;
pub mod single_flight;
pub mod task_queue;
pub mod time;
pub mod tracing;

pub(crate) use tracing::record;

pub fn format_stuid(stuid: &str) -> String {
    stuid.trim().to_uppercase()
}

/// 递归回溯 `Error::source` 链，生成形如：
/// ```text
/// OuterError(...)
/// caused by: MidError(...)
/// caused by: RootError(...)
/// ```
/// 的文本（每层用 `{:?}`）。
pub fn debug_error_chain(err: &dyn std::error::Error) -> String {
    let mut lines = vec![format!("{err:?}")];
    let mut current = err.source();
    while let Some(e) = current {
        lines.push(format!("caused by: {e:?}"));
        current = e.source();
    }
    lines.join("\n")
}
