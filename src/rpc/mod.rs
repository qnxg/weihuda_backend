mod user;

use crate::config::CFG;
use std::net::SocketAddr;
use user::UserServer;
use volo_gen::weihuda::rpc::UserServiceServer;
use volo_thrift::codec::default::{
    DefaultMakeCodec, framed::MakeFramedCodec,
    thrift::MakeThriftCodec,
};

pub async fn serve() {
    let address = match CFG.server.rpc_address.parse::<SocketAddr>() {
        Ok(address) => address,
        Err(e) => {
            tracing::error!(
                "Failed to parse RPC address {}: {:?}",
                &CFG.server.rpc_address,
                e
            );
            std::process::exit(1);
        }
    };

    tracing::info!("Starting RPC on {}", &CFG.server.rpc_address);
    let make_codec = DefaultMakeCodec::new(
        MakeFramedCodec::new(MakeThriftCodec::default())
            .with_max_frame_size(1024 * 1024),
    );
    if let Err(e) = UserServiceServer::new(UserServer)
        .make_codec(make_codec)
        .run(volo::net::Address::from(address))
        .await
    {
        tracing::error!("RPC server stopped: {:?}", e);
        std::process::exit(1);
    }
}
