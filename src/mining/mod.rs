use tokio::sync::oneshot;

pub mod mgr;
pub mod miner;
pub mod stats;


#[derive(Debug)]
pub enum MiningEvent {
    NewWork(epoch_number, epoch_challenge, address),
    SubmitResult(bool, Option<String>),
    Exit(oneshot::Sender<()>),
}

