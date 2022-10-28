

use std::{net::SocketAddr,
          sync::{atomic::Ordering, Arc},
          time::Duration,
};
use std::collections::VecDeque;
use std::hint::spin_loop;
use std::sync::atomic::{AtomicBool, AtomicU32};
use anyhow::Result;
use anyhow::{anyhow, bail};
use log::{debug, error, info};
use rand::{thread_rng, RngCore};
use json_rpc_types::Id;
use rayon::{ThreadPool, ThreadPoolBuilder};
use snarkvm::prelude::{Address, Testnet3, ToBytes};

use tokio::{
    net::TcpStream,
    net::TcpListener,
    task,
    time::{sleep, timeout},
    sync::{mpsc::channel},
};
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::{Receiver, Sender};
use crate::mining::stats::Stats;
use crate::stratum::message::StratumMessage;


pub struct Miner {
    pool: ThreadPool,
    miner_sender: Sender<MinerEvent>,
    miner_receiver: Receiver<MinerEvent>,
    stats: Arc<Stats>,
}

#[derive(Debug)]
pub enum MinerEvent {
    NewWork(u64, Option<String>),
    Exit(oneshot::Sender<()>),
}


impl Miner {
    pub fn new(index: u8, threads: u16, stats: Arc<Stats>,) -> Self {
        let (miner_sender, miner_receiver) = channel::<MinerEvent>(256);

        let pool = ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(threads as usize)
            .thread_name(move |idx| format!("miner-{}-{}", index, idx))
            .build().unwrap();

        Miner {
            pool,
            miner_sender,
            miner_receiver,
            stats,
        }
    }

    pub fn miner_sender(&self) -> Sender<MinerEvent> {
        self.miner_sender.clone()
    }

    pub fn start(&mut self, )  {
        // task::spawn(async move {
        //     while let Some(msg) = self.miner_receiver.recv().await {
        //         match msg {
        //             MinerEvent::NewWork(difficulty, message) => self.new_work().await,
        //             MinerEvent::Exit(responder) => {
        //                 self.wait_for_terminator();
        //                 responder.send(()).expect("failed response exit msg");
        //                 break;
        //             }
        //         }
        //     }
        // });
    }

    fn wait_for_terminator(&self) {
        //self.terminator.store(true, Ordering::SeqCst);
        // while !self.ready.load(Ordering::SeqCst) {
        //     spin_loop();
        // }
        //self.terminator.store(false, Ordering::SeqCst);
    }

    async fn new_work(&self) {
        //let height = template.block_height();
        // debug!("starting new work: {}", height);
        // self.wait_for_terminator();
        // debug!("the thread pool is ready to go");
        //
        // let terminator = self.terminator.clone();
        // let ready = self.ready.clone();
        // let statistic_router = self.statistic_router.clone();
        // let client_router = self.client_router.clone();
        //
        // let miner = self.clone();
        //
        // self.pool.spawn(move || {
        //     ready.store(false, Ordering::SeqCst);
        //     // ensure new work starts before returning
        //
        //     while !terminator.load(Ordering::SeqCst) {
        //         info!("Miner is mining now");
        //         //miner.stats.update_total_proofs();
        //     }
        //     debug!("block {} terminated", height);
        //     ready.store(true, Ordering::SeqCst);
        // });

        debug!("spawned new work");
    }

    async fn prove(epoch_challenge: EpochChallenge<Testnet3>, address: Address<Testnet3>) {
        let epoch_challenge = epoch_challenge.clone();
        let address = address.clone();
        let tp = tp.clone();
        let proving_key = proving_key.clone();
        if epoch_number != current_epoch.load(Ordering::SeqCst) {
            debug!(
                                        "Terminating stale work: current {} latest {}",
                                        epoch_number,
                                        current_epoch.load(Ordering::SeqCst)
                                    );
            return;
        }

        let nonce = thread_rng().next_u64();
        if let Ok(Ok(solution)) = task::spawn_blocking(move || {
            tp.install(|| {
                CoinbasePuzzle::prove(&proving_key.clone(), &epoch_challenge, &address, nonce)
            })
        })
            .await
        {
            if epoch_number != current_epoch.load(Ordering::SeqCst) {
                debug!(
                                            "Terminating stale work: current {} latest {}",
                                            epoch_number,
                                            current_epoch.load(Ordering::SeqCst)
                                        );
                return;
            }
            // Ensure the share difficulty target is met.
            let proof_difficulty =
                u64::MAX / sha256d_to_u64(&*solution.commitment().to_bytes_le().unwrap());
            if proof_difficulty < proof_target {
                debug!(
                                            "Share difficulty target not met: {} > {}",
                                            proof_difficulty, proof_target
                                        );
                total_proofs.fetch_add(1, Ordering::SeqCst);
                return;
            }

            info!(
                                        "Share found for epoch {} with difficulty {}",
                                        epoch_number, proof_difficulty
                                    );

            // Send a `PoolResponse` to the operator.
            let message = StratumMessage::Submit(
                Id::Num(0),
                client.address.to_string(),
                hex::encode(epoch_number.to_le_bytes()),
                hex::encode(nonce.to_bytes_le().unwrap()),
                hex::encode(solution.commitment().to_bytes_le().unwrap()),
                hex::encode(solution.proof().to_bytes_le().unwrap()),
            );
            if let Err(error) = client.sender().send(message).await {
                error!("Failed to send PoolResponse: {}", error);
            }
            //total_proofs.fetch_add(1, Ordering::SeqCst);
        }
    }



}