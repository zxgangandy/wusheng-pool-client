

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
use rayon::{ThreadPool, ThreadPoolBuilder};

use tokio::{
    net::TcpStream,
    net::TcpListener,
    task,
    time::{sleep, timeout},
    sync::{mpsc::channel},
};
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::Sender;
use crate::mining::stats::Stats;
use crate::stats::Stats;


pub struct Miner {
    pool: ThreadPool,
    //total_proofs: Arc<AtomicU32>,
    //running: AtomicBool,
    //stats: Stats,
    stats: Arc<Stats>,
}

#[derive(Debug)]
pub enum MinerEvent {
    NewWork(u64, Option<String>),
    Exit(oneshot::Sender<()>),
}


impl Miner {
    pub fn new(index: u8, threads: u16, stats: Arc<Stats>,) -> Self {
        //let core_threads = num_cpus::get() as u16 / max_core;

        // for index in 0..max_core {
        //     let pool = ThreadPoolBuilder::new()
        //         .stack_size(8 * 1024 * 1024)
        //         .num_threads(core_threads as usize)
        //         .thread_name(move |idx| format!("cpu-core-{}-{}", index, idx))
        //         .build().unwrap();
        //     thread_pools.push(Arc::new(pool));
        // }

        let pool = ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(threads as usize)
            .thread_name(move |idx| format!("miner-{}-{}", index, idx))
            .build().unwrap();

        Miner {
            pool,
            //total_proofs: Arc::new(Default::default()),
            //running: AtomicBool::new(false),
            //stats: Stats::new(),
            stats,
        }
    }

    fn running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub async fn stop(&self) {
        if self.running() {
            // let sender = self.prover_router.read().await;
            // let (tx, rx) = oneshot::channel();
            // if let Err(err) = sender.send(ProverMsg::Exit(tx)).await {
            //     error!("failed to stop prover: {err}");
            // }
            // rx.await.unwrap();
            debug!("miner exited");
            self.running.store(false, Ordering::SeqCst);
        }
    }

    fn _start(
        prover_router: Sender<ProverMsg>,
        statistic_router: Sender<StatisticMsg>,
        client_router: Sender<ClientMsg>,
        gpu_index: i16,
        threads: u8,
    ) -> Sender<WorkerMsg> {
        let (tx, mut rx) = mpsc::channel(100);
        let worker = Worker {
            pool: ThreadPoolBuilder::new().num_threads(threads as usize).build().unwrap(),
            terminator: Arc::new(AtomicBool::new(false)),
            ready: Arc::new(AtomicBool::new(true)),
            prover_router,
            statistic_router,
            client_router,
        };

        task::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    WorkerMsg::Notify(template, diff) => worker.new_work(template, diff, gpu_index).await,
                    WorkerMsg::Exit(responder) => {
                        worker.wait_for_terminator();
                        responder.send(()).expect("failed response exit msg");
                        break;
                    }
                }
            }
        });
        tx
    }

    fn wait_for_terminator(&self) {
        self.terminator.store(true, Ordering::SeqCst);
        while !self.ready.load(Ordering::SeqCst) {
            spin_loop();
        }
        self.terminator.store(false, Ordering::SeqCst);
    }

    async fn new_work(self: &Arc<Self>, template: Arc<BlockTemplate<Testnet2>>, share_difficulty: u64, gpu_index: i16) {
        let height = template.block_height();
        debug!("starting new work: {}", height);
        self.wait_for_terminator();
        debug!("the thread pool is ready to go");

        let terminator = self.terminator.clone();
        let ready = self.ready.clone();
        let statistic_router = self.statistic_router.clone();
        let client_router = self.client_router.clone();

        let miner = self.clone();

        self.pool.spawn(move || {
            ready.store(false, Ordering::SeqCst);
            // ensure new work starts before returning

            while !terminator.load(Ordering::SeqCst) {
                info!("Miner is mining now");
                //miner.stats.update_total_proofs();
            }
            debug!("block {} terminated", height);
            ready.store(true, Ordering::SeqCst);
        });

        debug!("spawned new work");
    }




}