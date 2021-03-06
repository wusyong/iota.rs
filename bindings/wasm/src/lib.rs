use wasm_bindgen::prelude::*;
use iota::crypto::Kerl;
use iota::signing::{IotaSeed, Seed};
use iota::ternary::{T1B1Buf, TryteBuf};
use iota::bundle::{Address, TransactionField};
use iota::client::response::Transfer;
use iota_conversion::Trinary;
use iota_bundle_preview::{Hash, Transaction};
use serde::{Serialize, Deserialize};

#[wasm_bindgen]
extern {
  #[wasm_bindgen(js_namespace = console)]
  pub fn log(s: &str);
  #[wasm_bindgen(js_namespace = console)]
  pub fn error(s: &str);
}

macro_rules! console_log {
  ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

macro_rules! console_error {
  ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
#[derive(Serialize)]
pub struct NewAddress {
    pub index: u64,
    address: String,
}

#[wasm_bindgen]
impl NewAddress {
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> String {
        self.address.clone()
    }
}

#[wasm_bindgen]
#[derive(Deserialize)]
pub struct NewTransfer {
    value: u64
}

#[wasm_bindgen]
#[derive(Serialize)]
pub struct SentTransaction {
    #[serde(rename = "isTail")]
    is_tail: bool
}

#[wasm_bindgen]
pub struct Client {
    client: iota::Client,
}

fn response_to_js_value<T: Serialize>(response: T) -> Result<JsValue, JsValue> {
    JsValue::from_serde(&response)
        .map_err(js_error)
}

fn js_error<T: std::fmt::Debug>(e: T) -> JsValue {
    JsValue::from(format!("{:?}", e))
}

fn create_hash(bytes: &[i8]) -> Hash {
    let mut array = [0; 243];
    let bytes = &bytes[..array.len()];
    array.copy_from_slice(bytes); 
    Hash(array)
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(uri: &str) -> Result<Client, JsValue> {
        console_error_panic_hook::set_once();
        let client = Client {
            client: iota::Client::new(uri)
                .map_err(|e| e.to_string())?
        };
        Ok(client)
    }

    #[wasm_bindgen(js_name = "getNodeInfo")]
    pub async fn get_node_info(self) -> Result<JsValue, JsValue> {
        let node_info = self.client.get_node_info()
            .await
            .map_err(|e| JsValue::from(e.to_string()))?;
        let res = response_to_js_value(node_info)?;
        Ok(res)
    }

    #[wasm_bindgen(js_name = "getNewAddress")]
    pub async fn get_new_address(self, seed: String, index: Option<u64>, security: Option<u8>) -> Result<JsValue, JsValue> {
        let encoded_seed = IotaSeed::<Kerl>::from_buf(
            TryteBuf::try_from_str(&seed)
            .map_err(js_error)?
            .as_trits()
            .encode::<T1B1Buf>(),
        )
            .map_err(js_error)?;

        let mut builder = self.client.get_new_address();
        builder = builder.seed(&encoded_seed);

        if let Some(index) = index {
            builder = builder.index(index);
        }
        if let Some(security) = security {
            builder = builder.security(security);
        }

        let (index, address) = builder
            .generate()
            .await
            .map_err(js_error)?;

        let new_address = NewAddress {
            index,
            address: address
                .to_inner()
                .as_i8_slice()
                .trytes()
                .map_err(js_error)?
        };
        let res = response_to_js_value(new_address)?;

        Ok(res)
    }

    #[wasm_bindgen(js_name = "addNeighbors")]
    pub async fn add_neighbors(self, uris: JsValue) -> Result<JsValue, JsValue> {
        let uris: Vec<String> = uris.into_serde().map_err(js_error)?;
        let builder = self.client.add_neighbors()
            .uris(uris)
            .map_err(js_error)?;

        let added_neighbords = builder
            .send()
            .await
            .map_err(js_error)?;
        let res = response_to_js_value(added_neighbords)?;

        Ok(res)
    }

    #[wasm_bindgen(js_name = "attachToTangle")]
    pub async fn attach_to_tangle(
        self,
        trunk_transaction_hash_bytes: JsValue,
        branch_transaction_hash_bytes: JsValue,
        min_weight_magnitude: Option<u8>,
        transactions_trytes: JsValue,
    ) -> Result<JsValue, JsValue> {
        let mut builder = self.client.attach_to_tangle();

        if trunk_transaction_hash_bytes.is_truthy() {
            let hash_vec: Vec<i8> = trunk_transaction_hash_bytes.into_serde().map_err(js_error)?;
            let hash = create_hash(&hash_vec[..]);
            builder = builder.trunk_transaction(&hash);
        }

        if branch_transaction_hash_bytes.is_truthy() {
            let hash_vec: Vec<i8> = branch_transaction_hash_bytes.into_serde().map_err(js_error)?;
            let hash = create_hash(&hash_vec[..]);
            builder = builder.branch_transaction(&hash);
        }

        if transactions_trytes.is_truthy() {
            // let transactions_trytes: Vec<Transaction> = transactions_trytes.into_serde().map_err(js_error)?;
        }

        if let Some(min_weight_magnitude) = min_weight_magnitude {
            builder = builder.min_weight_magnitude(min_weight_magnitude);
        }

        let attach_response = builder
            .send()
            .await
            .map_err(js_error)?;

        // TODO this needs impl Serialize on bee > bundle > Transaction
        // let response = response_to_js_value(&attach_response)?;

        Ok(JsValue::from(""))
    }

    #[wasm_bindgen(js_name = "broadcastBundle")]
    pub async fn broadcast_bundle(self, tail_transaction_hash_bytes: JsValue) -> Result<JsValue, JsValue> {
        let tail_transaction_hash_vec: Vec<i8> = tail_transaction_hash_bytes.into_serde().map_err(js_error)?;
        let tail_transaction_hash = create_hash(&tail_transaction_hash_vec);

        let broadcast_response = self.client.broadcast_bundle(&tail_transaction_hash)
            .await
            .map_err(js_error)?;
        
        // TODO this needs impl Serialize on bee > bundle > Transaction
        // let response = response_to_js_value(&broadcast_response)?;

        Ok(JsValue::from(""))
    }

    #[wasm_bindgen(js_name = "checkConsistency")]
    pub async fn check_consistency(self, tails: JsValue) -> Result<JsValue, JsValue> {
        let tails_vec: Vec<Vec<i8>> = tails.into_serde().map_err(js_error)?;
        let mut tails = Vec::new();
        for tail_vec in tails_vec {
            tails.push(create_hash(&tail_vec));
        }

        let builder = self.client.check_consistency()
            .tails(&tails);

        let consistency_response = builder
            .send()
            .await
            .map_err(js_error)?;

        let response = response_to_js_value(consistency_response)?;

        Ok(response)
    }

    #[wasm_bindgen(js_name =  "sendTransfers")]
    pub async fn send_transfers(self, seed: String, transfers: JsValue, min_weight_magnitude: Option<u8>) -> Result<JsValue, JsValue> {
        let encoded_seed = IotaSeed::<Kerl>::from_buf(
            TryteBuf::try_from_str(&seed)
            .map_err(js_error)?
            .as_trits()
            .encode::<T1B1Buf>(),
        )
            .map_err(js_error)?;

        let address = Address::from_inner_unchecked(
            TryteBuf::try_from_str(&seed)
            .map_err(js_error)?
            .as_trits()
            .encode(),
        );

        let js_transfers: Vec<NewTransfer> = transfers.into_serde().map_err(js_error)?;
        let transfers = js_transfers.iter().map(|transfer| Transfer {
            address: address.clone(),
            value: transfer.value,
            message: None,
            tag: None,
        }).collect();

        let mut builder = self.client.send_transfers()
            .seed(&encoded_seed)
            .transfers(transfers);

        if let Some(min_weight_magnitude) = min_weight_magnitude {
            builder = builder.min_weight_magnitude(min_weight_magnitude);
        }

        let transactions = builder
            .send()
            .await
            .map_err(js_error)?;

        let response: Vec<SentTransaction> = transactions.iter().map(|transaction| SentTransaction {
            is_tail: transaction.is_tail()
        }).collect();
        
        let res = response_to_js_value(response)?;
        Ok(res)
    }
}
