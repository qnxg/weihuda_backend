use opentelemetry::trace::TraceContextExt;

/// 向当前 span 记录一个或多个字段。
///
/// 这是对 [`tracing::record_all!`] 的薄封装，自动作用于 [`tracing::Span::current()`]。
/// 字段必须在 span 创建时预先声明（可用 [`tracing::field::Empty`] 占位），
/// 否则赋值不会生效。
///
/// 语法与 [`tracing::span!`] / [`tracing::info!`] 一致，支持以下可选格式化前缀：
///
/// - 无前缀：按 [`tracing::Value`] 默认规则记录
/// - `%`：使用 [`std::fmt::Display`]
/// - `?`：使用 [`std::fmt::Debug`]
///
/// # Examples
///
/// ```ignore
/// // span 创建时已声明 cache_result = tracing::field::Empty
/// utils::record!(cache_result = "hit");
///
/// utils::record!(
///     http.response.status_code = 200,
///     http.response.status_class = %"2xx",
/// );
/// ```
///
/// [`tracing::record_all!`]: tracing::macro@record_all
/// [`tracing::Span::current()`]: tracing::Span::current
macro_rules! record {
    ($($fields:tt)*) => {
        ::tracing::record_all!(::tracing::Span::current(), $($fields)*)
    };
}

pub(crate) use record;

/// 取当前活动 OTLP span 的 trace_id/span_id，用于将异步任务关联回触发它的请求。
/// 没有活动 span（非请求上下文）时返回 None。
pub fn current_trace_context() -> Option<(String, String)> {
    let span_context = opentelemetry::Context::current()
        .span()
        .span_context()
        .clone();
    if span_context.is_valid() {
        Some((
            span_context.trace_id().to_string(),
            span_context.span_id().to_string(),
        ))
    } else {
        None
    }
}
