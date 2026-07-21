use ahnlich_db::engine::{operations, store::StoreHandler};
use ahnlich_types::db::{query, server};
use prost::Message;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct AhnlichDB {
    handler: StoreHandler,
}

#[wasm_bindgen]
impl AhnlichDB {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AhnlichDB {
        let write_flag = Arc::new(AtomicBool::new(false));
        let handler = StoreHandler::new(Arc::clone(&write_flag));
        AhnlichDB { handler }
    }

    pub fn create_store(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::CreateStore::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        operations::create_store(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = server::Unit {};
        Ok(response.encode_to_vec())
    }

    pub fn create_pred_index(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::CreatePredIndex::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        operations::create_pred_index(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = server::Unit {};
        Ok(response.encode_to_vec())
    }

    pub fn create_non_linear_algorithm_index(
        &self,
        request_bytes: &[u8],
    ) -> Result<Vec<u8>, JsValue> {
        let params = query::CreateNonLinearAlgorithmIndex::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        operations::create_non_linear_algorithm_index(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = server::Unit {};
        Ok(response.encode_to_vec())
    }

    pub fn drop_pred_index(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::DropPredIndex::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        operations::drop_pred_index(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = server::Unit {};
        Ok(response.encode_to_vec())
    }

    pub fn drop_non_linear_algorithm_index(
        &self,
        request_bytes: &[u8],
    ) -> Result<Vec<u8>, JsValue> {
        let params = query::DropNonLinearAlgorithmIndex::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        operations::drop_non_linear_algorithm_index(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = server::Unit {};
        Ok(response.encode_to_vec())
    }

    pub fn set(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::Set::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        let result = operations::set(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = server::Set {
            upsert: Some(result),
        };
        Ok(response.encode_to_vec())
    }

    pub fn upsert(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::Upsert::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        let result = operations::upsert(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = server::Set {
            upsert: Some(result),
        };
        Ok(response.encode_to_vec())
    }

    pub fn del_key(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::DelKey::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        let deleted = operations::del_key(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = server::Del {
            deleted_count: deleted as u64,
        };
        Ok(response.encode_to_vec())
    }

    pub fn del_pred(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::DelPred::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        let deleted = operations::del_pred(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = server::Del {
            deleted_count: deleted as u64,
        };
        Ok(response.encode_to_vec())
    }

    pub fn drop_store(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::DropStore::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        let deleted = operations::drop_store(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = server::Del {
            deleted_count: deleted as u64,
        };
        Ok(response.encode_to_vec())
    }

    pub fn drop_schema(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::DropSchema::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        operations::drop_schema(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let response = server::Unit {};
        Ok(response.encode_to_vec())
    }

    pub fn get_key(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::GetKey::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        let result = operations::get_key(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(result.encode_to_vec())
    }

    pub fn get_pred(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::GetPred::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        let result = operations::get_pred(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(result.encode_to_vec())
    }

    pub fn get_sim_n(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::GetSimN::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        let result = operations::get_sim_n(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(result.encode_to_vec())
    }

    pub fn get_store(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::GetStore::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        let result = operations::get_store(&self.handler, params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(result.encode_to_vec())
    }

    pub fn list_stores(&self, request_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
        let params = query::ListStores::decode(request_bytes)
            .map_err(|e| JsValue::from_str(&format!("Decode error: {}", e)))?;

        let result = operations::list_stores(&self.handler, params);

        Ok(result.encode_to_vec())
    }

    pub fn export_snapshot(&self) -> Result<Vec<u8>, JsValue> {
        let stores = self.handler.get_stores();
        rmp_serde::to_vec(&ahnlich_db::engine::versioned::VersionedDbStores::current(
            stores,
        ))
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    pub fn import_snapshot(&mut self, snapshot_bytes: &[u8]) -> Result<(), JsValue> {
        let versioned: ahnlich_db::engine::versioned::VersionedDbStores =
            rmp_serde::from_slice(snapshot_bytes)
                .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;

        let stores = versioned
            .into_latest()
            .map_err(|e| JsValue::from_str(&format!("Migration error: {}", e)))?;

        self.handler.use_snapshot(stores);
        Ok(())
    }
}

impl Default for AhnlichDB {
    fn default() -> Self {
        Self::new()
    }
}
