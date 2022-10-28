use tokio::sync::oneshot;

pub mod mgr;
pub mod miner;
pub mod stats;


#[derive(Debug)]
pub enum MiningEvent {
    NewWork(u64, String, String),
    SubmitResult(bool, Option<String>),
    Exit(oneshot::Sender<()>),
}

