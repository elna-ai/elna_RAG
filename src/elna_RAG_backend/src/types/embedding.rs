// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call::CallResult as Result;

#[derive(CandidType, Deserialize)]
pub enum Result_ {
    Ok,
    Err(String),
}

pub struct Service(pub Principal);
impl Service {
    pub async fn clear_model_bytes(&self) -> Result<()> {
        ic_cdk::call(self.0, "clear_model_bytes", ()).await
    }
    pub async fn clear_vocab_bytes(&self) -> Result<()> {
        ic_cdk::call(self.0, "clear_vocab_bytes", ()).await
    }
    pub async fn get_embeddings(&self, arg0: String) -> Result<(Vec<Vec<f32>>,)> {
        ic_cdk::call(self.0, "get_embeddings", (arg0,)).await
    }
    pub async fn setup(&self) -> Result<(Result_,)> {
        ic_cdk::call(self.0, "setup", ()).await
    }
}
