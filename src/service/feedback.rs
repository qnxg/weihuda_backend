use crate::{
    error::AppResult,
    infra::{self, rabbitmq::RabbitMessage},
};

pub use infra::mysql::feedback::FeedbackInfo;
pub use infra::mysql::feedback::get_feedback;
pub use infra::mysql::feedback::get_feedback_list;
pub use infra::mysql::feedback::get_feedback_msg;

pub async fn add_feedback(
    desc: &str,
    contact: Option<&String>,
    img_url: Option<&String>,
    stu_id: Option<&str>,
) -> AppResult<u64> {
    let id = infra::mysql::feedback::add_feedback(
        desc, contact, img_url, stu_id,
    )
    .await?;
    let msg = RabbitMessage::Feedback {
        stu_id: stu_id.unwrap_or("未登录").to_string(),
        desc: desc.to_string(),
        img_url: img_url.cloned(),
        id,
    };
    if let Err(e) = infra::rabbitmq::publish_message(msg).await {
        tracing::warn!(
            error = ?e,
            "反馈消息投递到 RabbitMQ 失败，已忽略"
        );
    }
    Ok(id)
}
