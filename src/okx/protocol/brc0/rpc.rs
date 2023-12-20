use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub data: ZeroData,
}

#[derive(Debug, PartialEq, Clone,Deserialize, Serialize)]
pub struct ZeroData {
    pub block_height: String,
    pub block_hash: String,
    pub prev_block_hash: String,
    pub block_time: String,
    pub txs: Vec<BRCZeroTx>,
}

#[derive(Debug, PartialEq, Clone,Serialize,Deserialize)]
pub struct BRCZeroTx {
    pub protocol_name: String,
    pub inscription: String,
    pub inscription_context: String,
    pub btc_txid: String,
    pub btc_fee: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: Vec<TxResult>,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxResult {
    pub hash: String,
}