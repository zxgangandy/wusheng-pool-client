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

pub struct Client{
}

impl Client {
    pub fn new() -> Self {
        Client {}
    }

    pub async fn start(&self, pool_server: &SocketAddr, address: &Address<Testnet2>) -> Result<()>  {
        let stream = self.connect_to_pool_server(pool_server).await?;
        let mut framed = Framed::new(stream, StratumCodec::default());

        // step2. subscribe
        if let Err(error) = self.start_subscribe(&mut framed).await {
            error!("[Subscribe] {}", error);
            return Err(anyhow!("Failed to subscribe to the server {}", error));
        }

        Ok(())
    }


    pub async fn connect_to_pool_server(
        &self,
        pool_server: &SocketAddr,
    ) -> Result<TcpStream>  {
        loop {
            match timeout(Duration::from_secs(5), TcpStream::connect(pool_server)).await {
                Ok(stream) => match stream {
                    Ok(stream) => {
                        info!("Connected to {}", &pool_server);
                        return Ok(stream);
                    }
                    Err(e) => {
                        error!("Failed to connect to the pool server: {}", e);
                        sleep(Duration::from_secs(5)).await;
                    }
                },
                Err(_) => {
                    error!("Failed to connect to the pool server: Timed out");
                }
            }
        }
    }

    async fn start_subscribe(&self, framed: &mut Framed<TcpStream, StratumCodec>)-> Result<()> {

        if let Err(error) = self.send_subscribe_req(framed).await {
            error!("[Subscribe request] {}", error);
            return Err(anyhow!(error));
        }

        if let Err(error) = self.wait_subscribe_resp(framed).await {
            error!("[Subscribe response] {}", error);
            return Err(anyhow!(error));
        }

        Ok(())
    }

    async fn send_subscribe_req(&self, framed: &mut Framed<TcpStream, StratumCodec>)-> Result<()> {
        if let Err(error) = framed.send(StratumProtocol::Subscribe(
            Id::Num(0),
            format!("pool-client-{}", env!("CARGO_PKG_VERSION")),
            "AleoStratum/1.0.0".to_string(),
            None,
        )).await {
            return Err(anyhow!("Send subscribe request to the server error {}", error));
        }

        Ok(())
    }

    async fn wait_subscribe_resp(&self, framed: &mut Framed<TcpStream, StratumCodec>)-> Result<()> {
        match framed.next().await {
            Some(res) => match res {
                Ok(msg) => match msg {
                    StratumProtocol::Response(_id, _result, error) => {
                        if !error.is_none() {
                            error!("Client subscribe error {}", error.unwrap().message);
                            return Err(anyhow!("Server subscribe response with error!!!"));
                        } else {
                            info!("Client subscribe successful!!!");
                        }
                    }
                    _ => {
                        error!("Unexpected response for message {}", msg.name());
                        return Err(anyhow!("Unexpected response for message {}", msg.name()));
                    },
                },
                Err(e) => {
                    error!("{}", e);
                }
            },
            None => {
                error!("disconnected");
            }
        }
        info!("subscribe ok");
        Ok(())
    }
}

