mod stratum;
mod cmd;
mod client;
mod worker;
mod stats;

use structopt::StructOpt;
use log::{info};
use log4rs;
use cmd::gen_account;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(flatten)]
    cmd: Cmd,
}

#[derive(Debug, StructOpt)]
pub enum Cmd {
    GenAccount(gen_account::Cmd),
}

fn main() {
    log4rs::init_file("config/config.yaml", Default::default()).unwrap();
    info!("Hello, world!");
}
