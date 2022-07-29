use json_rpc_types::{Error, Id, Request, Response, Version};

/// Miner and pool server communication protocol
pub enum StratumProtocol {
    /// (id, user_agent, protocol_version, session_id)
    Subscribe(Id, String, String, Option<String>),

    /// (id, account_name, miner_name, worker_password)
    Authorize(Id, String, String, Option<String>),

    /// This is the difficulty target for the next job.
    /// (difficulty_target)
    SetTarget(u64),

    /// New job from the mining pool.
    /// (job_id, block_header_root, hashed_leaves_1, hashed_leaves_2, hashed_leaves_3,
    ///  hashed_leaves_4, clean_jobs)
    Notify(String, String, String, String, String, String, bool),

    /// Submit shares to the pool.
    /// (id, job_id, nonce, proof)
    Submit(Id, String, String, String),

    ///// (id, result, error)
    //Response(Id, Option<ResponseMessage>, Option<Error<()>>),
}