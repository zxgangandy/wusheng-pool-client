
use anyhow::Result;
use anyhow::{anyhow, bail};
use snarkvm::{
    dpc::testnet2::Testnet2,
    prelude::{BlockHeader, BlockTemplate},
};


pub struct Worker{

}


impl Worker {
    pub fn new() -> Self {
        Worker {}
    }

    pub fn start() -> Result<()> {
        //BlockHeader::mine_once_unchecked()
        Ok(())
    }

}