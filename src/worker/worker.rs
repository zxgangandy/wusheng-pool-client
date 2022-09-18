use std::{net::SocketAddr,
          sync::{atomic::Ordering, Arc},
          time::Duration,
};
use anyhow::Result;
use anyhow::{anyhow, bail};
// use snarkvm::{
//     dpc::testnet2::Testnet2,
//     prelude::{BlockHeader, BlockTemplate},
// };
//use snarkos::{Prover, ProverTrial};

use tokio::{
    net::TcpStream,
    net::TcpListener,
    task,
    time::{sleep, timeout},
    sync::{mpsc::channel},
};


pub struct Worker{
    pub pool_server: SocketAddr,
}


impl Worker {
    pub fn new(pool_server: SocketAddr) -> Self {
        Worker {pool_server}
    }

    pub fn start() -> Result<()> {
        //BlockHeader::mine_once_unchecked()

        task::spawn(async move {
            // Notify the outer function that the task is ready.
            //let _ = router.send(());
            // Asynchronously wait for a prover request.
            // while let Some(request) = prover_handler.recv().await {
            //     // Hold the prover write lock briefly, to update the state of the prover.
            //     prover.update(request).await;
            // }
        });

        Ok(())
    }

    // async fn process_pool_request(
    //     pool_server: SocketAddr,
    //     operator_ip: SocketAddr,
    //     share_difficulty: u64,
    //     block_template: BlockTemplate<N>
    // ) -> Result<()> {
    //
    //     if pool_server == operator_ip {
    //
    //     }
    //
    //
    //     Ok(())
    // }

    // fn get_worker_status()-> bool {
    //     return !Prover<Testnet2>::terminator().load(Ordering::SeqCst) &&
    //         !Prover<Testnet2>::status().is_peering() &&
    //         !Prover<Testnet2>::status().is_mining();
    // }

}