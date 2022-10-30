use structopt::lazy_static::lazy_static;
use tokio::sync::mpsc::Sender;
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

// lazy_static! {
//     static ref PROVER_SENDER: Mutex<Option<Sender>> = Mutex::new(Default::default());
// }

//pub static mut PROVER_SENDER: Option<Sender<ProverEvent>> = None;