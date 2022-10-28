use json_rpc_types::{Error, Id};
use serde::{Deserialize, Serialize};
use erased_serde::Serialize as ErasedSerialize;


/// Miner and pool server communication protocol
pub enum StratumMessage {
    /// (id, user_agent, protocol_version, session_id)
    Subscribe(Id, String, String, Option<String>),

    /// (id, miner_name, worker_password)
    Authorize(Id, String, String),

    /// This is the difficulty target for the next job.
    /// (difficulty_target)
    SetDifficulty(u64),

    /// New job from the mining pool.
    /// (job_id, block_header_root, hashed_leaves_1, hashed_leaves_2, hashed_leaves_3,
    ///  hashed_leaves_4, clean_jobs)
    Notify(String, String, String, String, String, String, bool),

    /// Submit shares to the pool.
    /// (id, worker_name, job_id, nonce, commitment, proof)
    Submit(Id, String, String, String, String, String),

    /// (result， message)
    Response(Id, Option<ResponseParams>, Option<Error<()>>),
}

impl StratumMessage {
    pub fn name(&self) -> &'static str {
        match self {
            StratumMessage::Subscribe(..) => "mining.subscribe",
            StratumMessage::Authorize(..) => "mining.authorize",
            StratumMessage::SetDifficulty(..) => "mining.set_difficulty",
            StratumMessage::Notify(..) => "mining.notify",
            StratumMessage::Submit(..) => "mining.submit",
            StratumMessage::Response(..) => "mining.response",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NotifyParams(pub String, pub String, pub String, pub String, pub String, pub String, pub bool);

#[derive(Serialize, Deserialize)]
pub struct SubscribeParams(pub String, pub String, pub Option<String>);

pub enum ResponseParams {
    Bool(bool),
    Array(Vec<Box<dyn ErasedSerialize + Send + Sync>>),
    Null,
}
