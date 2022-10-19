

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
use tokio::sync::mpsc::{Receiver, Sender};
use crate::mining::stats::Stats;


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




}