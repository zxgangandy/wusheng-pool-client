use tokio::sync::oneshot;

pub mod prover;
pub mod worker;
pub mod stats;


#[derive(Debug)]
pub enum ProverEvent {
    NewWork(u64, String, String),
    SubmitResult(bool, Option<String>),
    Exit(oneshot::Sender<()>),
}

