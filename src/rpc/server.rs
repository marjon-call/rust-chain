use axum::{routing::post, Router};
use axum::extract::{Json, State, DefaultBodyLimit};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::chain::blockchain::Blockchain;
use serde::{Serialize, Deserialize};

const MAX_REQUEST_BYTES: usize = 1024 * 128;

pub type SharedBlockchain = Arc<Mutex<Blockchain>>;

#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    pub method: String,
    pub params: Vec<serde_json::Value>,
    pub id: u64,
}

#[derive(Debug, Serialize)]
pub struct RpcResponse {
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}


// creates server w/ axum
pub async fn start(blockchain: SharedBlockchain, port: u16) -> Result<(), String> {
    let app = Router::new()
        .route("/", post(handle_rpc))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES)) // limits size
        .with_state(blockchain);

    let addr = format!("0.0.0.0:{}", port);
    println!("RPC server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await
        .map_err(|e| e.to_string())?;
        
    axum::serve(listener, app).await
        .map_err(|e| e.to_string())?;

    Ok(())
}

async fn handle_rpc(
    State(blockchain): State<SharedBlockchain>,
    Json(req): Json<RpcRequest>,
) -> impl axum::response::IntoResponse{

    let result: Result<serde_json::Value, String> = match req.method.as_str() {

        // queries block number
        "getBlockNumber" => {
            let chain = blockchain.lock().await;
            let height = chain.blocks.len() as u64;
            Ok(serde_json::json!(height))
        }

        // queries user balance
        "getBalance" => {
            let address = match req.params.get(0).and_then(|v| v.as_str()) {
                Some(a) => a.to_string(),
                None => return Json(RpcResponse { id: req.id, result: None, error: Some("RPC: missing address param".to_string()) }),
            };
            let chain = blockchain.lock().await;
            let balance = chain.state.get_balance(&address);
            Ok(serde_json::json!(balance))
        }

        // queries block data by number
        "getBlockByNumber" => {
            let index = match req.params.get(0).and_then(|v| v.as_u64()) {
                Some(i) => i as usize,
                None => return Json(RpcResponse { id: req.id, result: None, error: Some("RPC: missing block number param".to_string()) }),
            };
            let chain = blockchain.lock().await;
            match chain.blocks.get(index) {
                Some(block) => Ok(serde_json::json!(block)),
                None => Err("RPC:: block not found".to_string()),
            }
        }

        // sends real tranasction to the chain
        "sendRawTransaction" => {

            // get params
            let tx_bytes = match req.params.get(0).and_then(|v| v.as_str()) {
                Some(t) => t.to_string(),
                None => return Json(RpcResponse { id: req.id, result: None, error: Some("RPC: missing tx parameter".to_string())}),
            };

            // decode hex
            let bytes = match hex::decode(&tx_bytes) {
                Ok(b) => b,
                Err(e) => return Json(RpcResponse { id: req.id, result: None, error: Some(format!("RPC: invalid hex: {}", e))}),
            };

            // deserialize transaction 
            let tx: crate::types::transaction::Transaction = match bincode::deserialize(&bytes) {
                Ok(t) => t,
                Err(e) => return Json(RpcResponse { id: req.id, result: None, error: Some(format!("RPC: invalid tx: {}", e)) }),
            };

            // // validate correct tx type
            // if tx.tx_type != crate::types::transaction::TxType::Transfer {
            //     return Json(RpcResponse { id: req.id, result: None, error: Some("RPC: not a transfer transaction".to_string()) });
            // }

            // submit tx to chain
            let mut chain = blockchain.lock().await;
            match chain.submit_tx(tx) {
                Ok(_) => Ok(serde_json::json!("tx submitted to mempool")),
                Err(e) => Err(e),
            }

        }

        "getValidators" => {
            let chain = blockchain.lock().await;
            let validators: Vec<_> = chain.state.get_active_validators()
                .iter()
                .map(|v| serde_json::json!({
                    "address": v.address,
                    "stake": v.stake,
                    "is_active": v.is_active,
                    "last_proposed": v.last_proposed,
                }))
                .collect();

            Ok(serde_json::json!(validators))
        }

        _ => Err(format!("RPC: method not found: {}", req.method)),
    };

    match result {
        Ok(val) => Json(RpcResponse { id: req.id, result: Some(val), error: None }),
        Err(e) => Json(RpcResponse { id: req.id, result: None, error: Some(e) }),
    }

}