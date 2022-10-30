mod stratum;
mod cmd;
mod mining;
mod account;
mod utils;
mod storage;

use std::process;
use structopt::StructOpt;
use log::{error, info};
use log4rs;
use anyhow::Result;
use cmd::gen_account;
use cmd::cpu_mining;
use cmd::gpu_mining;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(flatten)]
    cmd: Cmd,
}

#[derive(Debug, StructOpt)]
pub enum Cmd {
    GenAccount(gen_account::Cmd),
    CpuMining(cpu_mining::Cmd),
    GpuMining(gpu_mining::Cmd),
}

async fn run(cli: Cli) -> Result<()> {
    match cli.cmd {
        Cmd::GenAccount(cmd) => cmd.run().await,
        Cmd::CpuMining(cmd) => cmd.run().await,
        Cmd::GpuMining(cmd) => cmd.run().await,
    }
}

#[tokio::main]
async fn main() -> Result<()>{
    log4rs::init_file("config/config.yaml", Default::default()).unwrap();

    let cli = Cli::from_args();

    if let Err(e) = run(cli).await {
        error!("Run command error: {:?}", e);
        process::exit(1);
    }

    std::future::pending::<()>().await;
    Ok(())
}
