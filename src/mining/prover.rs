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

use crate::mining::worker::{Worker, WorkerEvent};
use crate::mining::ProverEvent;
use crate::mining::stats::{Stats, StatsEvent};
use crate::stratum::message::StratumMessage;
use crate::utils::global;

pub struct Prover {
    workers: Vec<Sender<WorkerEvent>>,
    prover_sender: Option<Sender<ProverEvent>>,
    stats: Arc<Stats>,
    senders: Arc<global::Senders>,
    miner_address: String,
}

impl Prover {

    pub async fn new(senders: Arc<global::Senders>, miner_address: String,) -> Self {

        Self {
            workers: vec![],
            prover_sender: None,
            stats: Stats::new().await,
            senders,
            miner_address,
        }
    }

    pub async fn stop(&self) {
        if self.running() {
            let (tx, rx) = oneshot::channel();
            let prover_sender = self.prover_sender.clone().unwrap();
            if let Err(err) = prover_sender.send(ProverEvent::Exit(tx)).await {
                error!("failed to stop prover: {err}");
            }
            rx.await.unwrap();
            info!("Mgr exited");
        }
    }

    pub async fn start_cpu(
        mut self,
        num_miner: u8,
        address: impl ToString,
        pool_ip: SocketAddr,
    ) -> Result<()> {
        let (mgr_sender, mgr_receiver) = channel::<ProverEvent>(256);
        self.prover_sender.replace(mgr_sender);
        let address = Address::from_str(&address.to_string()).context("invalid aleo address")?;
        let senders = self.senders.clone();
        self.start_all(num_miner, address, pool_ip, mgr_receiver, senders).await?;
        Ok(())
    }

    async fn start_all(
        mut self,
        num_miner: u8,
        address: Address<Testnet3>,
        pool_ip: SocketAddr,
        mut mgr_receiver: Receiver<ProverEvent>,
        senders: Arc<global::Senders>,
    ) -> Result<()> {
        let threads = num_cpus::get() as u16 / num_miner as u16;
        for index in 0..num_miner {
            let mut miner = Worker::new(
                index,
                threads,
                self.stats.clone(),
                senders.clone(),
                self.miner_address.clone()
            );
            self.workers.push(miner.worker_sender());
        }

        self.serve(mgr_receiver);
        info!("start all started");
        Ok(())
    }



    fn serve(mut self, mut prover_receiver: Receiver<ProverEvent>) {
        task::spawn(async move {
            while let Some(msg) = prover_receiver.recv().await {
                match msg {
                    ProverEvent::Exit(responder) => {
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

    fn process_msg(&mut self, msg: ProverEvent) -> Result<()> {
        match msg {
            ProverEvent::NewWork(epoch_number, epoch_challenge, address) => {
                for worker in self.workers.iter() {
                    let event = WorkerEvent::NewWork(
                        epoch_number,
                        epoch_challenge.clone(),
                        address.clone()
                    );
                    worker.try_send(event)?;
                }
            }
            ProverEvent::SubmitResult(valid, msg) => {
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
        let mgr_sender = self.prover_sender.clone().unwrap();
        mgr_sender.send(ProverEvent::Exit(tx)).await.context("client")?;
        rx.await.context("failed to get exit response of client")?;

        for (i, worker) in self.workers.iter().enumerate() {
            let (tx, rx) = oneshot::channel();
            worker.send(WorkerEvent::Exit(tx)).await.context("worker")?;
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
