use log::{error, info};
use anyhow::Result;
use anyhow::{anyhow, bail};
use json_rpc_types::Id;
use tokio_util::codec::Framed;
use tokio::net::TcpStream;
use futures_util::sink::SinkExt;
use tokio_stream::StreamExt;
use crate::stratum::codec::StratumCodec;
use crate::stratum::message::StratumMessage;

pub struct AuthorizeHandler;


impl AuthorizeHandler {

    pub async fn exec(
        framed: &mut Framed<TcpStream, StratumCodec>,
        address: &String
    )-> Result<()> {
        if let Err(error) = AuthorizeHandler::send_authorize_req(framed, address).await {
            error!("[Authorize request] {}", error);
            return Err(anyhow!(error));
        }

        if let Err(error) = AuthorizeHandler::wait_authorize_resp(framed).await {
            error!("[Authorize response] {}", error);
            return Err(anyhow!(error));
        }

        Ok(())
    }

    async fn send_authorize_req(
        framed: &mut Framed<TcpStream, StratumCodec>,
        address: &String
    )-> Result<()> {
        let mut id = 1;
        let authorization = StratumMessage::Authorize(
            Id::Num(id),
            address.clone(),
            "123456".to_string()
        );
        id += 1;
        if let Err(e) = framed.send(authorization).await {
            return Err(anyhow!("Error sending authorization: {}", e));
        } else {
            info!("Client sent authorization success!!!");
        }
        Ok(())
    }

    async fn wait_authorize_resp(framed: &mut Framed<TcpStream, StratumCodec>)-> Result<()> {
        match framed.next().await {
            None => {
                return Err(anyhow!("Unexpected end of stream"));
            }
            Some(Ok(message)) => match message {
                StratumMessage::Response(_, _, _) => {
                    info!("Authorization successful");
                }
                _ => {
                    return Err(anyhow!("Unexpected message: {:?}", message.name()));
                }
            },
            Some(Err(e)) => {
                return Err(anyhow!("Error receiving authorization: {}", e));
            }
        }
        Ok(())
    }

}