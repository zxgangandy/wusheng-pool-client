use std::net::SocketAddr;
use std::time::Duration;

use json_rpc_types::Id;
use tokio::{
    net::TcpStream,
    task,
    time::{sleep, timeout},
};
use tokio_util::codec::Framed;
use futures_util::sink::SinkExt;
use tokio_stream::StreamExt;
use snarkvm::dpc::testnet2::Testnet2;
use snarkvm::prelude::Address;
use log::{error, info};
use anyhow::Result;
use anyhow::{anyhow, bail};
use crate::stratum::codec::StratumCodec;
use crate::stratum::protocol::StratumProtocol;

pub struct SubscribeHandler;

impl SubscribeHandler {
    pub async fn apply(
        framed: &mut Framed<TcpStream, StratumCodec>
    )-> Result<()> {
        if let Err(error) = SubscribeHandler::send_subscribe_req(framed).await {
            error!("[Subscribe request] {}", error);
            return Err(anyhow!(error));
        }

        if let Err(error) = SubscribeHandler::wait_subscribe_resp(framed).await {
            error!("[Subscribe response] {}", error);
            return Err(anyhow!(error));
        }

        Ok(())
    }

    async fn send_subscribe_req(
        framed: &mut Framed<TcpStream, StratumCodec>
    )-> Result<()> {
        info!("[start send subscribe request]");
        if let Err(error) = framed.send(StratumProtocol::Subscribe(
            Id::Num(0),
            format!("client/{}", env!("CARGO_PKG_VERSION")),
            "AleoStratum/1.0.0".to_string(),
            None,
        )).await {
            return Err(anyhow!("Send subscribe request error {}", error));
        }

        Ok(())
    }

    async fn wait_subscribe_resp(
        framed: &mut Framed<TcpStream, StratumCodec>
    )-> Result<()> {
        info!("[start wait subscribe response]");

        match framed.next().await {
            Some(response) => match response {
                Ok(message) => match message {
                    StratumProtocol::Response(_id, _result, error) => {
                        if !error.is_none() {
                            return Err(anyhow!("Response with error {}", error.unwrap().message));
                        } else {
                            info!("Client subscribe successful!!!");
                        }
                    }
                    _ => {
                        return Err(anyhow!("Unexpected response for message {}", message.name()));
                    },
                },
                Err(e) => {
                    return Err(anyhow!("Server disconnected: {}", e));
                }
            },
            None => {
                return Err(anyhow!("Server disconnected for unexpected reason!!!"));
            }
        }
        Ok(())
    }
}