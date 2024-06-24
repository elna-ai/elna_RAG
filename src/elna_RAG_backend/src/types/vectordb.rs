// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call::CallResult as Result;
use std::fmt;

#[derive(CandidType, Deserialize)]
pub enum Error {
    UniqueViolation,
    DimensionMismatch,
    NotFound,
    Unauthorized(String),
}

#[derive(CandidType, Deserialize)]
pub enum Result_ {
    Ok,
    Err(Error),
}

#[derive(CandidType, Deserialize)]
pub enum Result1 {
    Ok(Vec<String>),
    Err(Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UniqueViolation => write!(f, "UniqueViolation"),
            Error::DimensionMismatch => write!(f, "DimensionMismatch"),
            Error::NotFound => write!(f, "NotFound"),
            Error::Unauthorized(msg) => write!(f, "Unauthorized {msg}"),
        }
    }
}

pub struct Service(pub Principal);
impl Service {
    pub async fn build_index(&self, arg0: String) -> Result<(Result_,)> {
        ic_cdk::call(self.0, "build_index", (arg0,)).await
    }
    pub async fn create_collection(&self, arg0: String, arg1: u64) -> Result<(Result_,)> {
        ic_cdk::call(self.0, "create_collection", (arg0, arg1)).await
    }
    pub async fn delete_collection(&self, arg0: String) -> Result<(Result_,)> {
        ic_cdk::call(self.0, "delete_collection", (arg0,)).await
    }
    pub async fn get_collections(&self) -> Result<(Result1,)> {
        ic_cdk::call(self.0, "get_collections", ()).await
    }
    pub async fn get_docs(&self, arg0: String) -> Result<(Result1,)> {
        ic_cdk::call(self.0, "get_docs", (arg0,)).await
    }
    pub async fn insert(
        &self,
        arg0: String,
        arg1: Vec<Vec<f32>>,
        arg2: Vec<String>,
        arg3: String,
    ) -> Result<(Result_,)> {
        ic_cdk::call(self.0, "insert", (arg0, arg1, arg2, arg3)).await
    }
    pub async fn query(&self, arg0: String, arg1: Vec<f32>, arg2: i32) -> Result<(Result1,)> {
        ic_cdk::call(self.0, "query", (arg0, arg1, arg2)).await
    }
}
