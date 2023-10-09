use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: RpcParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcParams {
    pub height: String,
    pub block_hash: String,
    pub is_confirmed: bool,
    pub txs: Vec<BRCZeroTx>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BRCZeroTx {
    pub hex_rlp_encode_tx: String,
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