use std::{net::SocketAddr,
          sync::{atomic::Ordering, Arc},
          time::Duration,
};
use std::collections::VecDeque;
use std::sync::atomic::AtomicU32;
use anyhow::Result;
use anyhow::{anyhow, bail};
use log::{debug, info};
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
    thread_pools: Arc<Vec<Arc<ThreadPool>>>,
    total_proofs: Arc<AtomicU32>,
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

        CpuMiner {
            thread_pools: Arc::new(thread_pools),
            total_proofs: Arc::new(Default::default()),
            valid_shares: Arc::new(Default::default()),
            invalid_shares: Arc::new(Default::default())
        }
    }

    /// initialize calculator
    pub async fn initialize(&self)->Result<()> {
        let total_proofs = self.total_proofs.clone();
        task::spawn(async move {
            let mut log = VecDeque::<u32>::from(vec![0; 60]);
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                let proofs = total_proofs.load(Ordering::SeqCst);
                log.push_back(proofs);
                let m1 = *log.get(59).unwrap_or(&0);
                let m5 = *log.get(55).unwrap_or(&0);
                let m15 = *log.get(45).unwrap_or(&0);
                let m30 = *log.get(30).unwrap_or(&0);
                let m60 = log.pop_front().unwrap_or_default();
                info!(
                    "{}",
                    Cyan.normal().paint(format!(
                        "Total proofs: {} (1m: {} p/s, 5m: {} p/s, 15m: {} p/s, 30m: {} p/s, 60m: {} p/s)",
                        proofs,
                        Self::calculate_proof_rate(proofs, m1, 1),
                        Self::calculate_proof_rate(proofs, m5, 5),
                        Self::calculate_proof_rate(proofs, m15, 15),
                        Self::calculate_proof_rate(proofs, m30, 30),
                        Self::calculate_proof_rate(proofs, m60, 60),
                    ))
                );
            }
        });
        debug!("Created proof rate calculator");


        Ok(())
    }

    fn calculate_proof_rate(now: u32, past: u32, interval: u32) -> Box<str> {
        if interval < 1 {
            return Box::from("---");
        }
        if now <= past || past == 0 {
            return Box::from("---");
        }
        let rate = (now - past) as f64 / (interval * 60) as f64;
        Box::from(format!("{:.2}", rate))
    }

    pub async fn result(&self, success: bool, msg: Option<String>) {
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