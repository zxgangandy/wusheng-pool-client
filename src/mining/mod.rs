use structopt::lazy_static::lazy_static;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

pub mod prover;
pub mod worker;
pub mod stats;


#[derive(Debug)]
pub enum ProverEvent {
    NewTarget(u64),
    NewWork(u64, String, String),
    SubmitResult(bool, Option<String>),
    Exit(oneshot::Sender<()>),
}
