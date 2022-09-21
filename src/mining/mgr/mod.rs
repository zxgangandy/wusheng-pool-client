use std::net::SocketAddr;
use std::str::FromStr;
use log::{info, error};
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::{Context, ensure, Result};
use anyhow::{anyhow, bail};
use snarkvm::prelude::Address;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::mining::miner::MinerEvent;
use crate::stats::Stats;

pub struct Manager {
    running: AtomicBool,
    num_miner: Vec<Sender<MinerEvent>>,
}

impl Manager {

    pub fn new() -> Self {
        //let (tx, _) = mpsc::channel(1);
        Self {
            running: AtomicBool::new(false),
            //prover_router: RwLock::new(tx),
            num_miner: vec![]
        }
    }

    pub async fn stop(&self) {
        if self.running() {
            // let sender = self.prover_router.read().await;
            // let (tx, rx) = oneshot::channel();
            // if let Err(err) = sender.send(ProverMsg::Exit(tx)).await {
            //     error!("failed to stop prover: {err}");
            // }
            // rx.await.unwrap();
            info!("prover exited");
            self.running.store(false, Ordering::SeqCst);
        }
    }

    pub async fn start_cpu(
        &self,
        pool_ip: SocketAddr,
        num_miner: u8,
        thread_per_worker: u8,
        name: String,
        address: impl ToString,
    ) -> Result<()> {
        let address = Address::from_str(&address.to_string()).context("invalid aleo address")?;
        ensure!(!self.running(), "prover is already running");

        let router = self._start_cpu(worker, thread_per_worker, address, name, pool_ip).await?;
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
        name: String,
        pool_ip: SocketAddr,
    ) -> Result<Sender<ProverMsg>> {
        //let (prover_router, rx) = mpsc::channel(100);
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

        //self.serve(rx, client_router, statistic_router);
        info!("prover-cpu started");
        Ok(prover_router)
    }


    fn serve(
        mut self,
        mut rx: Receiver<ProverMsg>,
        client_router: Sender<ClientMsg>,
        statistic_router: Sender<StatisticMsg>,
    ) {
        task::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    ProverMsg::Exit(responder) => {
                        if let Err(err) = self.exit(&client_router, &statistic_router).await {
                            error!("failed to exit: {err}");
                            // grace exit failed, force exit
                            process::exit(1);
                        }
                        responder.send(()).unwrap();
                        break;
                    }
                    _ => {
                        if let Err(err) = self.process_msg(msg, &statistic_router) {
                            error!("prover failed to process message: {err}");
                        }
                    }
                }
            }
        });
    }

    fn process_msg(&mut self, msg: ProverMsg, statistic_router: &Sender<StatisticMsg>) -> Result<()> {
        match msg {
            ProverMsg::NewWork(template, difficulty) => {
                let template = Arc::new(template);
                for worker in self.workers.iter() {
                    worker.try_send(WorkerMsg::Notify(template.clone(), difficulty))?;
                }
            }
            ProverMsg::SubmitResult(valid, msg) => {
                if let Err(err) = statistic_router.try_send(StatisticMsg::SubmitResult(valid, msg)) {
                    error!("failed to send submit result to statistic mod: {err}");
                }
            }
            _ => {
                warn!("unexpected msg");
            }
        }

        Ok(())
    }

}