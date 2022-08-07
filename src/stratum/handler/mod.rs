

pub mod subscribe;
pub mod authorize;

use tokio::{
    net::TcpStream,
    task,
    time::{sleep, timeout},
};
use log::{error, info};
use anyhow::Result;
use tokio_util::codec::Framed;
use crate::stratum::codec::StratumCodec;
use authorize::AuthorizeHandler;
use subscribe::SubscribeHandler;

#[derive(Debug)]
pub struct Handler {
    pub framed: Framed<TcpStream, StratumCodec>,
    pub address: String,
}

impl Handler {

    pub fn new(framed: Framed<TcpStream, StratumCodec>, address: String)-> Self {
        return Handler{ framed, address}
    }

    /// Step 1:
    /// Client will send 'subscribe' request message to the stratum server,
    /// then the stratum server send back 'subscribe' response;
    /// Step 2:
    pub async fn run(&mut self) -> Result<()> {
        if let Err(error) = SubscribeHandler::apply(&mut self.framed).await {
            error!("[Subscribe handler apply failed] {}", error);
        }

        if let Err(error) = AuthorizeHandler::apply(&mut self.framed).await {
            error!("[Authorize handler apply failed] {}", error);
        }

        Ok(())
    }

}