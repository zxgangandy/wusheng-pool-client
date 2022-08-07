use log::{error, info};
use anyhow::Result;
use json_rpc_types::Id;
use tokio_util::codec::Framed;
use tokio::net::TcpStream;
use futures_util::sink::SinkExt;
use tokio_stream::StreamExt;
use crate::stratum::codec::StratumCodec;
use crate::stratum::protocol::StratumProtocol;

pub struct AuthorizeHandler;


impl AuthorizeHandler {

    async fn start_authorize(framed: &mut Framed<TcpStream, StratumCodec>)-> Result<()> {
        let mut id = 1;
        let authorization =
            StratumProtocol::Authorize(Id::Num(id), "client.address".to_string(), "".to_string());
        id += 1;
        if let Err(e) = framed.send(authorization).await {
            error!("Error sending authorization: {}", e);
        } else {
            info!("Sent authorization");
        }
        match framed.next().await {
            None => {
                error!("Unexpected end of stream");
                //sleep(Duration::from_secs(5)).await;
                //continue;
            }
            Some(Ok(message)) => match message {
                StratumProtocol::Response(_, _, _) => {
                    info!("Authorization successful");
                }
                _ => {
                    error!("Unexpected message: {:?}", message.name());
                }
            },
            Some(Err(e)) => {
                error!("Error receiving authorization: {}", e);
                //sleep(Duration::from_secs(5)).await;
                //continue;
            }
        }

        Ok(())

    }
}