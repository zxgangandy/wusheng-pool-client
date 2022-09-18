use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, prelude::*};
//use snarkvm::{dpc::testnet2::Testnet2, prelude::Account};
use log::{info, warn};
use structopt::StructOpt;
use anyhow::Result;
use snarkvm::prelude::Testnet3;
use crate::account::Account;


#[derive(Debug, StructOpt)]
/// Generating account command will generate ALEO account.
pub struct Cmd {
    ///The config file which stored the generated account.
    #[structopt(long)]
    file: Option<String>,
}

impl Cmd {

    pub async fn run(&self) -> Result<()> {
        info!("Start to run generating account command!!!");
        let account = Account::<Testnet3>::new()?;
        let private_key = format!("Private key:  {}", account.private_key());
        let view_key = format!("View key:  {}", account.view_key());
        let address = format!("Address:  {}", account.address());

        info!("{}", private_key);
        info!("{}", view_key);
        info!("{}", address);

        warn!("WARNING: Make sure you have a backup of both private key and view key!");
        warn!("WARNING: Nobody can help you recover those keys if you lose them!");

        if self.file.is_some() {
            let content = format!("{}\n{}\n{}", private_key, view_key, address);
            return self.backup_account(&content).await;
        }

        Ok(())
    }

    async fn backup_account(&self, content: &String)->Result<()> {
        let file = File::create(self.file.as_ref().unwrap())?;
        let mut writer = BufWriter::new(file);

        writer.write_all((content.to_string()).as_bytes()).expect("write failed");
        writer.flush().expect("flush failed");

        Ok(())
    }

}