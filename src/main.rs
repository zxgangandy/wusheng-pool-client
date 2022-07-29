mod stratum;
mod cmd;
mod client;
mod worker;
mod stats;

use std::process;
use structopt::StructOpt;
use log::{info};
use log4rs;
use anyhow::Result;
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

async fn run(cli: Cli) -> Result<()> {
    match cli.cmd {
        Cmd::GenAccount(cmd) => cmd.run().await,
    }
}

#[tokio::main]
async fn main() -> Result<()>{
    log4rs::init_file("config/config.yaml", Default::default()).unwrap();
    info!("Hello, world!");

    let cli = Cli::from_args();

    if let Err(e) = run(cli).await {
        eprintln!("error: {:?}", e);
        process::exit(1);
    }

    Ok(())
}
