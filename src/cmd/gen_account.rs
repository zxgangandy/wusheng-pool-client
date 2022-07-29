
use log::{info};
use structopt::StructOpt;
use anyhow::Result;


#[derive(Debug, StructOpt)]
/// Generating account command will generate ALEO account.
pub struct Cmd {
    ///The config file which stored the generated account.
    #[structopt(long)]
    file: String,
}

impl Cmd {

    pub async fn run(&self) -> Result<()> {
        info!("Start to run generating account command!!!");

        Ok(())
    }

}