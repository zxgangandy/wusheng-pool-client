use tokio::sync::oneshot;

pub mod mgr;
pub mod miner;

#[derive(Debug)]
pub enum MiningEvent {
    NewWork(u64, Option<String>),
    SubmitResult(bool, Option<String>),
    Exit(oneshot::Sender<()>),
}

