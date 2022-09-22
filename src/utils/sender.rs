use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use crate::mining::MiningEvent;
use crate::stratum::message::StratumMessage;

pub struct Wrapper {
    mgr_sender: Option<Sender<MiningEvent>>,
    handler_sender: Option<Sender<StratumMessage>>,
}

impl Wrapper {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            mgr_sender: None,
            handler_sender: None,
        })
    }

    pub fn set_mgr_sender(&mut self, mgr_sender: Sender<MiningEvent>) {
        self.mgr_sender.replace(mgr_sender);
    }

    pub fn mgr_sender(&self)->Sender<MiningEvent> {
        self.mgr_sender.unwrap()
    }

    pub fn set_handler_sender(&mut self, handler_sender: Sender<StratumMessage>) {
        self.handler_sender.replace(handler_sender);
    }

    pub fn handler_sender(&self) -> Sender<StratumMessage> {
        self.handler_sender.unwrap()
    }

}