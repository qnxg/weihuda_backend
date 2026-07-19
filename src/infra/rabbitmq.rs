use crate::{
    config::CFG,
    error::{AppResult, ThrowInternalErrorResult},
    utils,
};
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
        .await
        .internal_err()?;
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

#[tracing::instrument(
    skip_all
    fields(
        otel.kind = "client", 
        event_type = "mq", 
        exchange = tracing::field::Empty,
        routing_key = tracing::field::Empty,
    ),
    err
)]
pub async fn publish_message(msg: RabbitMessage) -> AppResult<()> {
    let channel = get_channel().await?;
    let exchange_name = match &msg {
        RabbitMessage::Feedback { .. } => {
            &CFG.rabbitmq.feedback_exchange
        }
    };
    utils::record!(exchange = %exchange_name);
    let routing_key = match msg {
        RabbitMessage::Feedback { .. } => "",
    };
    utils::record!(routing_key = routing_key);
    let payload = serde_json::to_vec(&msg).internal_err()?;
    channel
        .basic_publish(
            exchange_name,
            routing_key,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default(),
        )
        .await
        .internal_err()?
        .await
        .internal_err()?;
    Ok(())
}
