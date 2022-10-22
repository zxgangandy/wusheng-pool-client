use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use crate::mining::MiningEvent;
use crate::stratum::message::StratumMessage;

#[derive(Debug)]
pub struct Senders {
    mgr_sender: RwLock<Option<Sender<MiningEvent>>>,
    handler_sender: RwLock<Option<Sender<StratumMessage>>>,
}

impl Senders {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            mgr_sender: RwLock::new(None),
            handler_sender: RwLock::new(None),
        })
    }

    pub async fn set_mgr_sender(&self, mgr_sender: Sender<MiningEvent>) {
        *self.mgr_sender.write().await = Some(mgr_sender);
    }

    pub async fn mgr_sender(&self)->Sender<MiningEvent> {
        self.mgr_sender.read().await.clone().unwrap()
    }

    pub async fn set_handler_sender(&self, handler_sender: Sender<StratumMessage>) {
        *self.handler_sender.write().await = Some(handler_sender);
    }

    pub async fn handler_sender(&self) -> Sender<StratumMessage> {
        self.handler_sender.read().await.clone().unwrap()
    }

}