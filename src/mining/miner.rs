

use std::{net::SocketAddr,
          sync::{atomic::Ordering, Arc},
          time::Duration,
};
use std::collections::VecDeque;
use std::hint::spin_loop;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};
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
    stats: Arc<Stats>,
    terminator: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    current_epoch: Arc<AtomicU64>,
}

#[derive(Debug)]
pub enum MinerEvent {
    NewWork(epoch_number, epoch_challenge, address),
    Exit(oneshot::Sender<()>),
}


impl Miner {
    pub fn new(index: u8, threads: u16, stats: Arc<Stats>,) -> Arc<Self> {
        let (miner_sender, miner_receiver) = channel::<MinerEvent>(256);

        let pool = ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(threads as usize)
            .thread_name(move |idx| format!("miner-{}-{}", index, idx))
            .build().unwrap();

        let miner = Miner {
            pool,
            miner_sender,
            stats,
            terminator: Arc::new(AtomicBool::new(false)),
            ready: Arc::new(AtomicBool::new(true)),
            current_epoch: Default::default(),
        };

        let miner = Arc::new(miner);
        miner.clone().start(miner_receiver);

        miner
    }

    pub fn miner_sender(&self) -> Sender<MinerEvent> {
        self.miner_sender.clone()
    }

    pub fn start(self: Arc<Self>, mut miner_receiver: Receiver<MinerEvent>)  {
        let miner = Arc::clone(&self);

        task::spawn(async move {
            while let Some(msg) = miner_receiver.recv().await {
                match msg {
                    MinerEvent::NewWork(epoch_number, epoch_challenge, address) => {
                        let hex = &*hex::decode(epoch_challenge.as_bytes()).unwrap();
                        miner.new_work(
                            epoch_number,
                            EpochChallenge::<Testnet3>::from_bytes_le(hex).unwrap(),
                            Address::<Testnet3>::from_str(&address).unwrap(),
                        ).await
                    },
                    MinerEvent::Exit(responder) => {
                        miner.wait_for_terminator();
                        responder.send(()).expect("failed response exit msg");
                        break;
                    }
                }
            }
        });
    }

    fn wait_for_terminator(&self) {
        self.terminator.store(true, Ordering::SeqCst);
        while !self.ready.load(Ordering::SeqCst) {
            spin_loop();
        }
        self.terminator.store(false, Ordering::SeqCst);
    }

    async fn new_work(
        &self,
        epoch_number: u64,
        epoch_challenge: EpochChallenge<Testnet3>,
        address: Address<Testnet3>
    ) {
        debug!("starting new work: {}", epoch_number);
        self.wait_for_terminator();
        debug!("the thread pool is ready to go");
        //
        let terminator = self.terminator.clone();
        let ready = self.ready.clone();
        self.current_epoch.store(epoch_number, Ordering::SeqCst);
        let current_epoch = self.current_epoch.clone();

        self.pool.spawn(move || {
            ready.store(false, Ordering::SeqCst);
            // ensure new work starts before returning

            while !terminator.load(Ordering::SeqCst) {
                info!("Miner is mining now");
                //miner.stats.update_total_proofs();
                Self::prove(current_epoch, epoch_number, epoch_challenge, address).await;
            }

            debug!("block {} terminated", height);
            ready.store(true, Ordering::SeqCst);
        });

        debug!("spawned new work");
    }

    async fn prove(
        current_epoch: u64,
        epoch_number: u64,
        epoch_challenge: EpochChallenge<Testnet3>,
        address: Address<Testnet3>
    ) {
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
        }).await {
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
                debug!("Share difficulty target not met: {} > {}",
                    proof_difficulty,
                    proof_target
                );
                total_proofs.fetch_add(1, Ordering::SeqCst);
                return;
            }

            info!("Share found for epoch {} with difficulty {}",
                epoch_number,
                proof_difficulty
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