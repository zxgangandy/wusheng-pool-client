

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
use log::{error, info};
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
use crate::mining::MiningEvent;
use crate::stratum::message::{ResponseParams, StratumMessage};
use crate::utils::global::Senders;

#[derive(Debug)]
pub struct Handler {
    //pub framed: Framed<TcpStream, StratumCodec>,
    pub address: String,
    pub handler_sender: Sender<StratumMessage>,
    pub handler_receiver: Receiver<StratumMessage>,
    pub senders: Arc<Senders>,
    pub current_proof_target: Arc<AtomicU64>,
}

impl Handler {

    pub fn new(
        //framed: Framed<TcpStream, StratumCodec>,
        address: &String,
        senders: Arc<Senders>
    ) -> Self {
        let (handler_sender, mut handler_receiver) = channel::<StratumMessage>(1024);
        senders.set_handler_sender(handler_sender.clone());

        return Handler {
            //framed,
            address: address.clone(),
            handler_sender,
            handler_receiver,
            senders,
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

                let mut bool_result = false;
                let mut msg_result = "".to_string();
                match result.unwrap() {
                    ResponseParams::Bool(v)=>{
                        //submit_result = v;
                    },
                    _ => {}
                }

                // self.senders
                //     .mgr_sender()
                //     .send(MiningEvent::SubmitResult(submit_result, Some(msg_result)))
                //     .await
                //     .context("")?;
            }
            StratumMessage::Notify(
                job_id,
                block_header_root,
                hashed_leaves_1,
                hashed_leaves_2,
                hashed_leaves_3,
                hashed_leaves_4,
                _
            ) => {
                info!("Client received notify message");
                // self.senders
                //     .mgr_sender()
                //     .send(MiningEvent::NewWork(0, Some("NewWork".to_string())))
                //     .await
                //     .context("failed to send notify to miner manager")?;
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