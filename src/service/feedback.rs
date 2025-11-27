use crate::{
    infra::{self, rabbitmq::RabbitMessage},
    result::AppResult,
};

pub use infra::mysql::feedback::FeedbackInfo;
pub use infra::mysql::feedback::get_feedback_list;

pub async fn add_feedback(
    typ: &str,
    desc: &str,
    contact: Option<&String>,
    img_url: Option<&String>,
    stu_id: &str,
) -> AppResult<u64> {
    let id = infra::mysql::feedback::add_feedback(
        typ, desc, contact, img_url, stu_id,
    )
    .await?;
    let msg = RabbitMessage::Feedback {
        stu_id: stu_id.to_string(),
        desc: desc.to_string(),
        img_url: img_url.cloned(),
        id,
    };
    infra::rabbitmq::publish_message(msg).await?;
    Ok(id)
}

pub use infra::mysql::feedback::update_feedback_status;
