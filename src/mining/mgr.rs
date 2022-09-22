use std::net::SocketAddr;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use log::{info, error, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::{Context, ensure, Result};
use anyhow::{anyhow, bail};
use snarkvm::prelude::Address;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{mpsc, oneshot};
use tokio::task;

use crate::mining::miner::MinerEvent;
use crate::mining::MiningEvent;
use crate::mining::stats::{Stats, StatsEvent};
use crate::stats::{Stats, StatsEvent};
use crate::stratum::message::StratumMessage;
use crate::utils::sender::Wrapper;

pub struct Manager {
    running: AtomicBool,
    workers: Vec<Sender<MinerEvent>>,
    mgr_sender: Sender<MiningEvent>,
    mgr_receiver: Receiver<MiningEvent>,
    stats: Arc<Stats>,
    wrapper: Arc<Wrapper>,
}

impl Manager {

    pub fn new(wrapper: Arc<Wrapper>, ) -> Self {
        let (mgr_sender, mgr_receiver) = channel::<MiningEvent>(256);

        Self {
            running: AtomicBool::new(false),
            workers: vec![],
            mgr_sender,
            mgr_receiver,
            stats: Arc::new(Stats::new()),
            wrapper,
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
        &self,
        pool_ip: SocketAddr,
        num_miner: u8,
        thread_per_worker: u8,
        address: impl ToString,
    ) -> Result<()> {
        let address = Address::from_str(&address.to_string()).context("invalid aleo address")?;
        ensure!(!self.running(), "prover is already running");

        let router = self._start_cpu(worker, thread_per_worker, address, pool_ip).await?;
        self.running.store(true, Ordering::SeqCst);
        let mut prover_router = self.prover_router.write().await;
        *prover_router = router;
        Ok(())
    }


    async fn _start_cpu(
        mut self,
        num_miner: u8,
        thread_per_worker: u8,
        address: Address<Testnet2>,
        pool_ip: SocketAddr,
    ) -> Result<Sender<ProverMsg>> {
        //let (mgr_sender, rx) = mpsc::channel(256);
        //let client_router = Client::start(pool_ip, prover_router.clone(), name, address);
        //let statistic_router = Statistic::start(client_router.clone());
        for _ in 0..num_miner {
            self.workers.push(Worker::start_cpu(
                prover_router.clone(),
                statistic_router.clone(),
                client_router.clone(),
                thread_per_worker,
            ));
        }

        info!(
            "created {} workers with {} threads each for the prover",
            self.workers.len(),
            thread_per_worker
        );

        self.serve(rx, client_router);
        info!("prover-cpu started");
        Ok(prover_router)
    }


    fn serve(
        mut self,
        mut rx: Receiver<ProverMsg>,
        client_router: Sender<ClientMsg>,
    ) {
        task::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    MiningEvent::Exit(responder) => {
                        if let Err(err) = self.exit(&client_router, &statistic_router).await {
                            error!("failed to exit: {err}");
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

    async fn exit(&mut self, client_router: &Sender<ClientMsg>, statistic_router: &Sender<StatisticMsg>) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        client_router.send(ClientMsg::Exit(tx)).await.context("client")?;
        rx.await.context("failed to get exit response of client")?;

        for (i, worker) in self.workers.iter().enumerate() {
            let (tx, rx) = oneshot::channel();
            worker.send(MinerEvent::Exit(tx)).await.context("worker")?;
            rx.await.context("failed to get exit response of worker")?;
            info!("worker {i} terminated");
        }
        let (tx, rx) = oneshot::channel();
        statistic_router
            .send(StatisticMsg::Exit(tx))
            .await
            .context("statistic")?;
        rx.await.context("failed to get exit response of statistic mod")?;
        Ok(())
    }

}