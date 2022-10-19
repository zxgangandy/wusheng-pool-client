use std::net::SocketAddr;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use log::{info, error, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::{Context, ensure, Result};
use anyhow::{anyhow, bail};
use snarkvm::prelude::{Address, Testnet3};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{mpsc, oneshot};
use tokio::task;

use crate::mining::miner::{Miner, MinerEvent};
use crate::mining::MiningEvent;
use crate::mining::stats::{Stats, StatsEvent};
use crate::stratum::message::StratumMessage;
use crate::utils::global;

pub struct Manager {
    running: AtomicBool,
    workers: Vec<Sender<MinerEvent>>,
    mgr_sender: Sender<MiningEvent>,
    mgr_receiver: Receiver<MiningEvent>,
    stats: Arc<Stats>,
    senders: Arc<global::Senders>,
}

impl Manager {

    pub fn new(senders: Arc<global::Senders>, ) -> Self {
        let (mgr_sender, mgr_receiver) = channel::<MiningEvent>(256);

        Self {
            running: AtomicBool::new(false),
            workers: vec![],
            mgr_sender,
            mgr_receiver,
            stats: Arc::new(Stats::new()),
            senders,
        }
    }

    pub async fn stop(&self) {
        if self.running() {
            let (tx, rx) = oneshot::channel();
            if let Err(err) = self.mgr_sender.send(MiningEvent::Exit(tx)).await {
                error!("failed to stop prover: {err}");
            }
            rx.await.unwrap();
            info!("prover exited");
            self.running.store(false, Ordering::SeqCst);
        }
    }

    pub async fn start_cpu(
        &mut self,
        num_miner: u8,
        address: impl ToString,
        pool_ip: SocketAddr,
    ) -> Result<()> {
        let address = Address::from_str(&address.to_string()).context("invalid aleo address")?;
        ensure!(!self.running(), "prover is already running");

        self._start_cpu(num_miner, address, pool_ip).await?;
        self.running.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }


    async fn _start_cpu(
        &mut self,
        num_miner: u8,
        address: Address<Testnet3>,
        pool_ip: SocketAddr,
    ) -> Result<()> {
        let threads = num_cpus::get() as u16 / num_miner as u16;
        for index in 0..num_miner {
            let mut miner = Miner::new(index, threads, self.stats.clone());
            self.workers.push(miner.miner_sender());
            miner.start();
        }

        self.serve();
        info!("start-cpu started");
        Ok(())
    }



    fn serve(&mut self, ) {
        let mgr_receiver = self.mgr_receiver;
        task::spawn(async move {
            while let Some(msg) = self.mgr_receiver.recv().await {
                match msg {
                    MiningEvent::Exit(responder) => {
                        if let Err(err) = self.exit().await {
                            error!("Failed to exit: {err}");
                            // grace exit failed, force exit
                            process::exit(1);
                        }
                        responder.send(()).unwrap();
                        break;
                    }
                    _ => {
                        if let Err(err) = self.process_msg(msg) {
                            error!("Miner manager failed to process message: {err}");
                        }
                    }
                }
            }
        });
    }

    fn process_msg(&mut self, msg: MiningEvent) -> Result<()> {
        match msg {
            MiningEvent::NewWork(..) => {
                for worker in self.workers.iter() {
                    worker.try_send(MinerEvent::NewWork(0, Some("NewWork".to_string())))?;
                }
            }
            MiningEvent::SubmitResult(valid, msg) => {
                if let Err(err) = self.stats.sender().try_send(
                    StatsEvent::SubmitResult(valid, msg)
                ) {
                    error!("Failed to send submit result to stats: {err}");
                }
            }
            _ => {
                warn!("Unexpected msg");
            }
        }

        Ok(())
    }

    async fn exit(&mut self, ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.mgr_sender.send(MiningEvent::Exit(tx)).await.context("client")?;
        rx.await.context("failed to get exit response of client")?;

        for (i, worker) in self.workers.iter().enumerate() {
            let (tx, rx) = oneshot::channel();
            worker.send(MinerEvent::Exit(tx)).await.context("worker")?;
            rx.await.context("failed to get exit response of worker")?;
            info!("worker {i} terminated");
        }
        let (tx, rx) = oneshot::channel();
        self.stats.sender()
            .send(StatsEvent::Exit(tx))
            .await
            .context("statistic")?;
        rx.await.context("failed to get exit response of statistic mod")?;
        Ok(())
    }
}
