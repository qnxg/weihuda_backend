use crate::{config::CFG, result::AppResult};
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties,
    options::BasicPublishOptions,
};
use serde::Serialize;
use tokio::sync::OnceCell;

static RABBIT_CHANNEL: OnceCell<Channel> = OnceCell::const_new();

pub async fn get_channel() -> Channel {
    RABBIT_CHANNEL
        .get_or_init(|| async {
            let conn = match Connection::connect(
                &CFG.rabbitmq.url,
                ConnectionProperties::default(),
            )
            .await
            {
                Ok(conn) => {
                    tracing::info!(
                        "🔥 Successfully connected to RabbitMQ"
                    );
                    conn
                }
                Err(e) => {
                    tracing::error!(
                        "🪨 Failed to connect to RabbitMQ: {:?}",
                        e
                    );
                    std::process::exit(1);
                }
            };
            match conn.create_channel().await {
                Ok(channel) => {
                    tracing::info!(
                        "🔥 Successfully create RabbitMQ channel"
                    );
                    channel
                }
                Err(e) => {
                    tracing::error!(
                        "🪨 Failed to create RabbitMQ channel: {:?}",
                        e
                    );
                    std::process::exit(1);
                }
            }
        })
        .await
        .clone()
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
    let channel = get_channel().await;
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
