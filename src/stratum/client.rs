use std::net::SocketAddr;
use std::time::Duration;

use tokio::{
    net::TcpStream,
    task,
    time::{sleep, timeout},
};
use tokio_util::codec::Framed;
use snarkvm::dpc::testnet2::Testnet2;
use snarkvm::prelude::Address;
use log::{error, info};
use anyhow::Result;

pub struct Client{

}

impl Client {

    pub async fn start(&self, pool_server: SocketAddr, address: Address<Testnet2>) -> Result<()>  {
        // task::spawn(async move {
        //
        //
        // });

        let stream = self.connect_to_pool_server(pool_server).await?;

        // step2. subscribe
        self.start_subscribe().await;

        Ok(());
    }


    async fn connect_to_pool_server(
        &self,
        pool_server: SocketAddr,
    ) -> Result<TcpStream>  {
        loop {
            match timeout(Duration::from_secs(5), TcpStream::connect(pool_server)).await {
                Ok(stream) => match stream {
                    Ok(stream) => {
                        info!("Connected to {}", &pool_server);
                        Ok(stream);
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

    async fn start_subscribe(&self,)-> Result<()> {

        self.send_subscribe_req().await;
        self.wait_subscribe_resp().await;
        Ok(());
    }

    async fn send_subscribe_req(&self,)-> Result<()> {

        Ok(());
    }

    async fn wait_subscribe_resp(&self,)-> Result<()> {

        Ok(());
    }
}

