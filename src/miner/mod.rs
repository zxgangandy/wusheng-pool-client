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
use crate::stats::Stats;


pub struct Miner {
    thread_pools: Arc<Vec<Arc<ThreadPool>>>,
    total_proofs: Arc<AtomicU32>,
    running: AtomicBool,
    stats: Stats,
}

#[allow(clippy::large_enum_variant)]
pub enum MinerEvent {
    NewWork(u64, Option<String>),
    Result(bool, Option<String>),
}


impl Miner {
    pub fn new(threads: u16,) -> Self {

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
                .build().unwrap();
            thread_pools.push(Arc::new(pool));
        }
        info!("Created {} thread pools with {} threads each", thread_pools.len(), pool_threads);

        Miner {
            thread_pools: Arc::new(thread_pools),
            total_proofs: Arc::new(Default::default()),
            running: AtomicBool::new(false),
            stats: Stats::new(),
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
                miner.stats.update_total_proofs();
            }
            debug!("block {} terminated", height);
            ready.store(true, Ordering::SeqCst);
        });

        debug!("spawned new work");
    }




}

