use std::{net::SocketAddr,
          sync::{atomic::Ordering, Arc},
          time::Duration,
};
use std::sync::atomic::AtomicU32;
use anyhow::Result;
use anyhow::{anyhow, bail};
use log::info;
use rayon::{ThreadPool, ThreadPoolBuilder};
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


pub struct CpuMiner {
    valid_shares: Arc<AtomicU32>,
    invalid_shares: Arc<AtomicU32>,
}


impl CpuMiner {
    pub fn new() -> Self {

        let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();
        let pool_count;
        let pool_threads;
        if threads % 12 == 0 {
            pool_count = threads / 12;
            pool_threads = 12;
        } else if threads % 10 == 0 {
            pool_count = threads / 10;
            pool_threads = 10;
        } else if threads % 8 == 0 {
            pool_count = threads / 8;
            pool_threads = 8;
        } else {
            pool_count = threads / 6;
            pool_threads = 6;
        }

        for index in 0..pool_count {
            let pool = ThreadPoolBuilder::new()
                .stack_size(8 * 1024 * 1024)
                .num_threads(pool_threads as usize)
                .thread_name(move |idx| format!("cpu-miner-{}-{}", index, idx))
                .build()?;
            thread_pools.push(Arc::new(pool));
        }
        info!("Created {} thread pools with {} threads each", thread_pools.len(), pool_threads);

        CpuMiner {}
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

    async fn result(&self, success: bool, msg: Option<String>) {
        if success {
            let valid_minus_1 = self.valid_shares.fetch_add(1, Ordering::SeqCst);
            let valid = valid_minus_1 + 1;
            let invalid = self.invalid_shares.load(Ordering::SeqCst);

            if let Some(msg) = msg {
                info!(
                    "{}",
                    Green.normal().paint(format!(
                        "Share accepted: {}  {} / {} ({:.2}%)",
                        msg,
                        valid,
                        valid + invalid,
                        (valid as f64 / (valid + invalid) as f64) * 100.0
                    ))
                );
            } else {
                info!(
                    "{}",
                    Green.normal().paint(format!(
                        "Share accepted  {} / {} ({:.2}%)",
                        valid,
                        valid + invalid,
                        (valid as f64 / (valid + invalid) as f64) * 100.0
                    ))
                );
            }
        } else {
            let invalid_minus_1 = self.invalid_shares.fetch_add(1, Ordering::SeqCst);
            let invalid = invalid_minus_1 + 1;
            let valid = self.valid_shares.load(Ordering::SeqCst);
            if let Some(msg) = msg {
                info!(
                    "{}",
                    Red.normal().paint(format!(
                        "Share rejected: {}  {} / {} ({:.2}%)",
                        msg,
                        valid,
                        valid + invalid,
                        (valid as f64 / (valid + invalid) as f64) * 100.0
                    ))
                );
            } else {
                info!(
                    "{}",
                    Red.normal().paint(format!(
                        "Share rejected  {} / {} ({:.2}%)",
                        valid,
                        valid + invalid,
                        (valid as f64 / (valid + invalid) as f64) * 100.0
                    ))
                );
            }
        }
    }

}