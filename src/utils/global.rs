use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use crate::mining::ProverEvent;
use crate::stratum::message::StratumMessage;

#[derive(Debug)]
pub struct Senders {
    prover_sender: RwLock<Option<Sender<ProverEvent>>>,
    handler_sender: RwLock<Option<Sender<StratumMessage>>>,
}

impl Senders {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            prover_sender: RwLock::new(None),
            handler_sender: RwLock::new(None),
        })
    }

    pub async fn set_prover_sender(&self, prover_sender: Sender<ProverEvent>) {
        *self.prover_sender.write().await = Some(prover_sender);
    }

    pub async fn prover_sender(&self) ->Sender<ProverEvent> {
        self.prover_sender.read().await.clone().unwrap()
    }

    pub async fn set_handler_sender(&self, handler_sender: Sender<StratumMessage>) {
        *self.handler_sender.write().await = Some(handler_sender);
    }

    pub async fn handler_sender(&self) -> Sender<StratumMessage> {
        self.handler_sender.read().await.clone().unwrap()
    }

}