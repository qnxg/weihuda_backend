mod user;

use crate::config::CFG;
use futures::{future, prelude::*};
use tarpc::{
    server::{self, Channel},
    tokio_serde::formats::Json,
};
use user::{User, UserServer};

pub async fn serve() {
    let mut listener = match tarpc::serde_transport::tcp::listen(
        &CFG.server.rpc_address,
        Json::default,
    )
    .await
    {
        Ok(listener) => {
            tracing::info!(
                "Successfully started RPC on {}",
                &CFG.server.rpc_address
            );
            listener
        }
        Err(e) => {
            tracing::error!("Failed to start RPC: {:?}", e);
            std::process::exit(1);
        }
    };
    listener.config_mut().max_frame_length(1024 * 1024);
    listener
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        .map(|channel| {
            channel.execute(UserServer.serve()).for_each(spawn)
        })
        .buffer_unordered(100)
        .for_each(|_| async {})
        .await;
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
