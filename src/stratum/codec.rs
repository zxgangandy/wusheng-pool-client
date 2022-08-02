use bytes::BytesMut;
use json_rpc_types::{Error, Id, Request, Response, Version};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;
use tokio_util::codec::{AnyDelimiterCodec, Decoder, Encoder};
use crate::stratum::protocol::StratumProtocol;

pub struct StratumCodec {
    pub codec: AnyDelimiterCodec,
}

impl Default for StratumCodec {
    fn default() -> Self {
        Self {
            codec: AnyDelimiterCodec::new_with_max_length(vec![b'\n'], vec![b'\n'], 4096),
        }
    }
}


impl Encoder<StratumProtocol> for StratumCodec {
    type Error = io::Error;

    fn encode(&mut self, item: StratumProtocol, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = match item {
            StratumProtocol::Subscribe(id, user_agent, protocol_version, session_id) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "mining.subscribe",
                    params: Some(SubscribeParams(user_agent, protocol_version, session_id)),
                    id: Some(id),
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumProtocol::Authorize(id, account_name, worker_name, worker_password) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "mining.authorize",
                    params: Some(AuthorizeParams(account_name, worker_name, worker_password)),
                    id: Some(id),
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumProtocol::SetTarget(difficulty_target) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "mining.set_target",
                    params: Some(vec![difficulty_target]),
                    id: None,
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumProtocol::Notify(
                job_id,
                block_header_root,
                hashed_leaves_1,
                hashed_leaves_2,
                hashed_leaves_3,
                hashed_leaves_4,
                clean_jobs,
            ) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "mining.notify",
                    params: Some(NotifyParams(
                        job_id,
                        block_header_root,
                        hashed_leaves_1,
                        hashed_leaves_2,
                        hashed_leaves_3,
                        hashed_leaves_4,
                        clean_jobs,
                    )),
                    id: None,
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumProtocol::Submit(id, job_id, nonce, proof) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "mining.submit",
                    params: Some(vec![job_id, nonce, proof]),
                    id: Some(id),
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumProtocol::Response(id, result, error) => match error {
                Some(error) => {
                    let response = Response::<(), ()>::error(Version::V2, error, Some(id));
                    serde_json::to_vec(&response).unwrap_or_default()
                }
                None => {
                    let response = Response::<Option<ResponseMessage>, ()>::result(
                        Version::V2,
                        result,
                        Some(id),
                    );
                    serde_json::to_vec(&response).unwrap_or_default()
                }
            },
        };
        let string = std::str::from_utf8(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        self.codec
            .encode(string, dst)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(())
    }
}

impl Decoder for StratumCodec {
    type Item = StratumProtocol;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let string = self
            .codec
            .decode(src)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        if string.is_none() {
            return Ok(None);
        }
        let bytes = string.unwrap();
        let json = serde_json::from_slice::<Value>(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        if !json.is_object() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Not an object"));
        }
        let object = json.as_object().unwrap();
        let result = if object.contains_key("method") {
            let request = serde_json::from_value::<Request<Vec<Value>>>(json)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            let id = request.id;
            let method = request.method.as_str();
            let params = match request.params {
                Some(params) => params,
                None => return Err(io::Error::new(io::ErrorKind::InvalidData, "No params")),
            };

            match method {
                "mining.subscribe" => {
                    if params.len() != 3 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let user_agent = unwrap_str_value(&params[0])?;
                    let protocol_version = unwrap_str_value(&params[1])?;
                    let session_id = match &params[2] {
                        Value::String(s) => Some(s),
                        Value::Null => None,
                        _ => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid params",
                            ))
                        }
                    };
                    StratumProtocol::Subscribe(
                        id.unwrap_or(Id::Num(0)),
                        user_agent,
                        protocol_version,
                        session_id.cloned(),
                    )
                }
                "mining.authorize" => {
                    if params.len() != 3 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let account_name = unwrap_str_value(&params[0])?;
                    let miner_name = unwrap_str_value(&params[1])?;
                    let worker_password = match &params[2] {
                        Value::String(s) => Some(s),
                        Value::Null => None,
                        _ => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid params",
                            ))
                        }
                    };
                    StratumProtocol::Authorize(
                        id.unwrap_or(Id::Num(0)),
                        account_name,
                        miner_name,
                        worker_password.cloned(),
                    )
                }
                "mining.set_target" => {
                    if params.len() != 1 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let difficulty_target = unwrap_u64_value(&params[0])?;
                    StratumProtocol::SetTarget(difficulty_target)
                }
                "mining.notify" => {
                    if params.len() != 7 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let job_id = unwrap_str_value(&params[0])?;
                    let block_header_root = unwrap_str_value(&params[1])?;
                    let hashed_leaves_1 = unwrap_str_value(&params[2])?;
                    let hashed_leaves_2 = unwrap_str_value(&params[3])?;
                    let hashed_leaves_3 = unwrap_str_value(&params[4])?;
                    let hashed_leaves_4 = unwrap_str_value(&params[5])?;
                    let clean_jobs = unwrap_bool_value(&params[6])?;

                    StratumProtocol::Notify(
                        job_id,
                        block_header_root,
                        hashed_leaves_1,
                        hashed_leaves_2,
                        hashed_leaves_3,
                        hashed_leaves_4,
                        clean_jobs,
                    )
                }
                "mining.submit" => {
                    if params.len() != 3 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }

                    let job_id = unwrap_str_value(&params[0])?;
                    let nonce = unwrap_str_value(&params[1])?;
                    let proof = unwrap_str_value(&params[2])?;

                    StratumProtocol::Submit(id.unwrap_or(Id::Num(0)), job_id, nonce, proof)
                }
                _ => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown method"));
                }
            }
        } else {
            let response = serde_json::from_value::<Response<ResponseMessage, ()>>(json)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            let id = response.id;
            match response.payload {
                Ok(payload) => {
                    StratumProtocol::Response(id.unwrap_or(Id::Num(0)), Some(payload), None)
                }
                Err(error) => StratumProtocol::Response(id.unwrap_or(Id::Num(0)), None, Some(error)),
            }
        };
        Ok(Some(result))
    }
}

fn unwrap_str_value(value: &Value) -> Result<String, io::Error> {
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Param is not str",
        )),
    }
}

fn unwrap_bool_value(value: &Value) -> Result<bool, io::Error> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Param is not bool",
        )),
    }
}

fn unwrap_u64_value(value: &Value) -> Result<u64, io::Error> {
    match value {
        Value::Number(n) => Ok(n
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Param is not u64"))?),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Param is not u64",
        )),
    }
}