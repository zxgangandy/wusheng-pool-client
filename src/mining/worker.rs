

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

use snarkvm::{
    compiler::{UniversalSRS},
    console::account::address::Address,
    prelude::{CoinbasePuzzle, Environment, FromBytes, Testnet3, ToBytes},
};
use snarkvm::algorithms::crypto_hash::sha256d_to_u64;
use snarkvm::compiler::{CoinbaseProvingKey, EpochChallenge, PuzzleConfig};

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
use crate::storage::Storage;


pub struct Worker {
    pool: ThreadPool,
    worker_sender: Sender<WorkerEvent>,
    stats: Arc<Stats>,
    terminator: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    current_epoch: Arc<AtomicU64>,
    current_proof_target: Arc<AtomicU64>,
    storage: Arc<Storage>,
    puzzle: CoinbasePuzzle<Testnet3>,
    miner_address: String,
}

#[derive(Debug)]
pub enum WorkerEvent {
    NewTarget(u64),
    NewWork(u64, String, String),
    Exit(oneshot::Sender<()>),
}


impl Worker {
    pub fn new(
        index: u8,
        threads: u16,
        stats: Arc<Stats>,
        storage: Arc<Storage>,
        miner_address: String,
    ) -> Arc<Self> {
        let (miner_sender, miner_receiver) = channel::<WorkerEvent>(256);

        let pool = ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(threads as usize)
            .thread_name(move |idx| format!("miner-{}-{}", index, idx))
            .build().unwrap();

        info!("Initializing universal SRS");
        let srs = UniversalSRS::<Testnet3>::load().expect("Failed to load SRS");
        let degree = (1 << 13) - 1;
        let config = PuzzleConfig { degree };
        let puzzle = CoinbasePuzzle::<Testnet3>::trim(&srs, config).unwrap();

        let miner = Worker {
            pool,
            worker_sender: miner_sender,
            stats,
            terminator: Arc::new(AtomicBool::new(false)),
            ready: Arc::new(AtomicBool::new(true)),
            current_epoch: Default::default(),
            current_proof_target: Default::default(),
            storage,
            puzzle,
            miner_address,
        };

        let miner = Arc::new(miner);
        miner.clone().start(miner_receiver);

        miner
    }

    pub fn worker_sender(&self) -> Sender<WorkerEvent> {
        self.worker_sender.clone()
    }

    pub fn start(self: Arc<Self>, mut worker_receiver: Receiver<WorkerEvent>)  {
        let miner = Arc::clone(&self);

        task::spawn(async move {
            let miner = Arc::clone(&miner);

            while let Some(msg) = worker_receiver.recv().await {
                match msg {
                    WorkerEvent::NewTarget(target) => {
                        miner.new_target(target);
                    }
                    WorkerEvent::NewWork(epoch_number, epoch_challenge, address) => {
                        let hex = &*hex::decode(epoch_challenge.as_bytes()).unwrap();
                        let stats = miner.stats.clone();
                        let storage = miner.storage.clone();
                        let miner_address = miner.miner_address.clone();
                        let puzzle = miner.puzzle.clone();

                        miner.new_work(
                            epoch_number,
                            EpochChallenge::<Testnet3>::from_bytes_le(hex).unwrap(),
                            Address::<Testnet3>::from_str(&address).unwrap(),
                            stats.clone(),
                            storage.clone(),
                            puzzle.clone(),
                            miner_address.clone(),
                        ).await
                    },
                    WorkerEvent::Exit(responder) => {
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

    fn new_target(&self, proof_target: u64) {
        self.current_proof_target.store(proof_target, Ordering::SeqCst);
        info!("New proof target: {}", proof_target);
    }

    async fn new_work(
        &self,
        epoch_number: u64,
        epoch_challenge: EpochChallenge<Testnet3>,
        address: Address<Testnet3>,
        stats: Arc<Stats>,
        storage: Arc<Storage>,
        puzzle: CoinbasePuzzle<Testnet3>,
        miner_address: String,
    ) {
        debug!("starting new work: {}", epoch_number);
        self.wait_for_terminator();
        debug!("the thread pool is ready to go");
        //
        let terminator = self.terminator.clone();
        let ready = self.ready.clone();
        self.current_epoch.store(epoch_number, Ordering::SeqCst);
        let current_epoch = self.current_epoch.clone();
        let proof_target = self.current_proof_target.load(Ordering::SeqCst);

        task::spawn(async move {
            ready.store(false, Ordering::SeqCst);

            while !terminator.load(Ordering::SeqCst) {
                info!("Miner is mining now");
                let current_epoch = current_epoch.clone();
                let epoch_challenge = epoch_challenge.clone();
                let stats = stats.clone();
                let storage = storage.clone();
                let puzzle = puzzle.clone();
                let miner_address = miner_address.clone();

                let result = Self::prove(
                    current_epoch,
                    epoch_number,
                    epoch_challenge,
                    proof_target,
                    address,
                    stats,
                    storage,
                    puzzle,
                    miner_address
                ).await?;

                if result {
                    break;
                }
            }

            //debug!("block {} terminated", height);
            ready.store(true, Ordering::SeqCst);
        });

        debug!("spawned new work");
    }

    async fn prove(
        current_epoch: Arc<AtomicU64>,
        epoch_number: u64,
        epoch_challenge: EpochChallenge<Testnet3>,
        proof_target: u64,
        address: Address<Testnet3>,
        stats: Arc<Stats>,
        storage: Arc<Storage>,
        puzzle: CoinbasePuzzle<Testnet3>,
        miner_address: String,
    ) -> Result<bool> {
        if epoch_number != current_epoch.load(Ordering::SeqCst) {
            debug!(
                "Terminating stale work: current {} latest {}",
                epoch_number,
                current_epoch.load(Ordering::SeqCst)
            );
            return Ok(true);
        }

        let nonce = thread_rng().next_u64();
        let stats = stats.clone();

        if let Ok(Ok(solution)) = task::spawn_blocking(move || {
            puzzle.prove(&epoch_challenge, address, nonce)
        }).await {
            if epoch_number != current_epoch.load(Ordering::SeqCst) {
                debug!(
                    "Terminating stale work: current {} latest {}",
                    epoch_number,
                    current_epoch.load(Ordering::SeqCst)
                );
                return Ok(true);
            }
            // Ensure the share difficulty target is met.
            let proof_difficulty = solution.to_target()?;
            //u64::MAX / sha256d_to_u64(&*solution.commitment().to_bytes_le().unwrap());
            if proof_difficulty < proof_target {
                debug!("Share difficulty target not met: {} > {}",
                    proof_difficulty,
                    proof_target
                );
                stats.update_total_proofs();
                return Ok(false);
            }

            info!("Share found for epoch {} with difficulty {}",
                epoch_number,
                proof_difficulty
            );

            let message = StratumMessage::Submit(
                Id::Num(0),
                miner_address,
                hex::encode(epoch_number.to_le_bytes()),
                hex::encode(nonce.to_bytes_le().unwrap()),
                hex::encode(solution.commitment().to_bytes_le().unwrap()),
                solution.to_string(),
            );
            if let Err(error) = storage.handler_sender().await.send(message).await {
                error!("Failed to send PoolResponse: {}", error);
            }

            stats.update_total_proofs();
        }

        return Ok(true);
    }

}