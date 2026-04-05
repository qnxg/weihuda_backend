use crate::{config::CFG, result::AppResult};
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties,
    options::BasicPublishOptions,
};
use serde::Serialize;
use tokio::sync::OnceCell;

static RABBIT_CHANNEL: OnceCell<Channel> = OnceCell::const_new();

pub async fn get_channel() -> AppResult<Channel> {
    let channel = RABBIT_CHANNEL
        .get_or_try_init(|| async {
            let conn = Connection::connect(
                &CFG.rabbitmq.url,
                ConnectionProperties::default(),
            )
            .await?;
            tracing::info!("🔥 Successfully connected to RabbitMQ");
            let channel = conn.create_channel().await?;
            tracing::info!("🔥 Successfully create RabbitMQ channel");
            Ok::<Channel, lapin::Error>(channel)
        })
        .await?;
    Ok(channel.clone())
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum RabbitMessage {
    Feedback {
        stu_id: String,
        desc: String,
        img_url: Option<String>,
        id: u64,
    },
}

pub async fn publish_message(msg: RabbitMessage) -> AppResult<()> {
    let channel = get_channel().await?;
    let exchange_name = match msg {
        RabbitMessage::Feedback { .. } => {
            &CFG.rabbitmq.feedback_exchange
        }
    };
    let routing_key = match msg {
        RabbitMessage::Feedback { .. } => "",
    };

    let payload = serde_json::to_vec(&msg)?;

    channel
        .basic_publish(
            exchange_name,
            routing_key,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default(),
        )
        .await?
        .await?;

    Ok(())
}
