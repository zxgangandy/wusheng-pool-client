

pub mod subscribe;
pub mod authorize;
pub mod notify;
pub mod set_target;

use tokio::{
    net::TcpStream,
    task,
    sync::{mpsc, oneshot},
    time::{sleep, timeout},
};
use log::{error, info};
use anyhow::Result;
use anyhow::{anyhow, bail};
use tokio_util::codec::Framed;
use futures_util::sink::SinkExt;
use snarkvm::utilities::error;
use tokio_stream::StreamExt;
use crate::stratum::codec::StratumCodec;
use authorize::AuthorizeHandler;
use subscribe::SubscribeHandler;
use crate::stratum::protocol::StratumProtocol;

#[derive(Debug)]
pub struct Handler {
    pub framed: Framed<TcpStream, StratumCodec>,
    pub address: String,
}

impl Handler {

    pub fn new(framed: Framed<TcpStream, StratumCodec>, address: &String)-> Self {
        return Handler{ framed, address: address.clone()}
    }

    /// handler run
    ///
    /// Step 1:
    /// Client will send 'subscribe' request message to the stratum server,
    /// then the stratum server send back 'subscribe' response;
    ///
    /// Step 2:
    /// Client will send 'authorize' request message to the stratum server,
    /// then the stratum server send back 'authorize' response;
    pub async fn run(&mut self) -> Result<()> {
        if let Err(error) = SubscribeHandler::apply(&mut self.framed).await {
            error!("[Subscribe handler apply failed] {}", error);
            return Err(anyhow!(error));
        }

        if let Err(error) = AuthorizeHandler::apply(&mut self.framed, &self.address).await {
            error!("[Authorize handler apply failed] {}", error);
            return Err(anyhow!(error));
        }

        let framed = &mut self.framed;
        let (net_router, mut net_handler) = mpsc::channel::<StratumProtocol>(1024);
        loop {
            tokio::select! {
                Some(message) = net_handler.recv() => {
                    let name = message.name();
                    if let Err(e) = framed.send(message).await {
                        error!("Client send failed {}: {:?}", name, e);
                        return Err(anyhow!(e));
                    }
                }
                result = framed.next() => match result {
                    Some(Ok(message)) => {
                        //TODO://
                        return Ok(());
                    }
                    Some(Err(e)) => {
                        error!("Client failed to read the message: {:?}", e);
                        return Err(anyhow!(e));
                    }
                    None => {
                        error!("Server disconnected!!!");
                        return Err(anyhow!("Server disconnected!!!"));
                    }
                }
            }
        }
    }

    async fn process_message(framed: &mut Framed<TcpStream, StratumCodec>) -> Result<()> {
        match framed.next().await {
            Some(Ok(message)) => {
                match message {
                    StratumProtocol::Response(_, result, error) => {
                        return Ok(());
                    }
                    StratumProtocol::Notify(
                        job_id,
                        block_header_root,
                        hashed_leaves_1,
                        hashed_leaves_2,
                        hashed_leaves_3,
                        hashed_leaves_4,
                        _
                    ) => {
                        return Ok(());
                    }
                    StratumProtocol::SetTarget(difficulty_target) => {
                        return Ok(());
                    }
                    _ => {
                        info!("Unhandled message: {}", message.name());
                    }
                }
            }
            Some(Err(e)) => {
                error!("Failed to read the message: {:?}", e);
            }
            None => {
                error!("Disconnected from server");
                // sleep(Duration::from_secs(5)).await;
                // break;
            }
        }

        Ok(())
    }

}