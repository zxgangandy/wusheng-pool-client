

pub mod subscribe;
pub mod authorize;
pub mod notify;
pub mod set_difficulty;

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::{
    net::TcpStream,
    task,
    sync::{mpsc, oneshot},
    time::{sleep, timeout},
};
use log::{debug, error, info};
use anyhow::{Context, Result};
use anyhow::{anyhow, bail};
use tokio_util::codec::Framed;
use futures_util::sink::SinkExt;
use snarkvm::utilities::error;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::StreamExt;
use crate::stratum::codec::StratumCodec;
use authorize::AuthorizeHandler;
use subscribe::SubscribeHandler;
use crate::mining::ProverEvent;
use crate::stratum::message::{ResponseParams, StratumMessage};
use crate::storage::Storage;

#[derive(Debug)]
pub struct Handler {
    //pub framed: Framed<TcpStream, StratumCodec>,
    pub address: String,
    pub handler_sender: Sender<StratumMessage>,
    pub handler_receiver: Receiver<StratumMessage>,
    pub storage: Arc<Storage>,
    pub current_proof_target: Arc<AtomicU64>,
}

impl Handler {

    pub fn new(
        //framed: Framed<TcpStream, StratumCodec>,
        address: &String,
        storage: Arc<Storage>
    ) -> Self {
        let (handler_sender, mut handler_receiver) = channel::<StratumMessage>(1024);

        return Handler {
            //framed,
            address: address.clone(),
            handler_sender,
            handler_receiver,
            storage,
            current_proof_target: Arc::new(Default::default())
        };
    }

    /// handler run
    ///
    /// Step 1:
    /// Client will send 'subscribe' request message to the stratum server,
    /// then the stratum server send back 'subscribe' response;
    ///
    /// Step 2:
    /// Client will send 'authorize' request message to the stratum server,
    /// then the stratum server send back 'authorize' response;
    pub async fn run(
        &mut self,
        //mgr_sender: Sender<MiningEvent>,
        framed: &mut Framed<TcpStream, StratumCodec>,
    ) -> Result<()> {
        if let Err(error) = SubscribeHandler::exec(framed).await {
            error!("[Subscribe handler apply failed] {}", error);
            return Err(anyhow!(error));
        }

        if let Err(error) = AuthorizeHandler::exec(framed, &self.address).await {
            error!("[Authorize handler apply failed] {}", error);
            return Err(anyhow!(error));
        }

        //let framed = &mut self.framed;
        loop {
            tokio::select! {
                Some(message) = self.handler_receiver.recv() => {
                    let name = message.name();
                    if let Err(e) = framed.send(message).await {
                        error!("Client send failed {}: {:?}", name, e);
                        return Err(anyhow!(e));
                    }
                }
                result = framed.next() => match result {
                    Some(Ok(message)) => {
                        self.process_mining_message(message, framed).await?;
                    }
                    Some(Err(e)) => {
                        error!("Client failed to read the message: {:?}", e);
                        return Err(anyhow!(e));
                    }
                    None => {
                        error!("Server disconnected!!!");
                        return Err(anyhow!("Server disconnected!!!"));
                    }
                }
            }
        }
    }

    async fn process_mining_message(
        &self,
        message: StratumMessage,
        framed: &mut Framed<TcpStream, StratumCodec>
    ) -> Result<()> {
        match message {
            StratumMessage::Response(_, result, error) => {
                info!("Client received response message");

                match result {
                    Some(params) => {
                        match params {
                            ResponseParams::Bool(result) => {
                                if result {
                                    if let Err(e) = self.storage.prover_sender()
                                        .await
                                        .send(ProverEvent::SubmitResult(result, None))
                                        .await
                                    {
                                        error!("Error sending share result to prover: {}", e);
                                    }
                                } else {
                                    error!("Unexpected result: {}", result);
                                }
                            }
                            _ => {
                                error!("Unexpected response params");
                            }
                        }
                    }
                    None => {
                        let error = error.unwrap();
                        let msg = Some(error.message.to_string());
                        if let Err(e) = self.storage.prover_sender()
                            .await
                            .send(ProverEvent::SubmitResult(false, msg))
                            .await
                        {
                            error!("Error sending share result to prover: {}", e);
                        }
                    }
                }
            }
            StratumMessage::Notify(job_id, epoch_challenge, address, _, ) => {
                info!("Client received notify message");
                let job_id_bytes = hex::decode(job_id).expect("Failed to decode job_id");
                if job_id_bytes.len() != 8 {
                    bail!("Unexpected job_id length: {}", job_id_bytes.len());
                }
                let epoch = u64::from_le_bytes(job_id_bytes[0..8].try_into().unwrap());

                self.storage
                    .prover_sender()
                    .await
                    .send(ProverEvent::NewWork(epoch, epoch_challenge, address.unwrap()))
                    .await
                    .context("failed to send notify to prover")?;
            }
            StratumMessage::SetDifficulty(difficulty_target) => {
                info!("Client received set target message");
                self.new_target(difficulty_target);
            }
            _ => {
                info!("Unexpected message: {}", message.name());
            }
        }

        Ok(())
    }

    fn new_target(&self, proof_target: u64) {
        self.current_proof_target.store(proof_target, Ordering::SeqCst);
        info!("New proof target: {}", proof_target);
    }

}