// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call::CallResult as Result;

#[derive(CandidType, Deserialize)]
pub enum DetailValue {
    I64(i64),
    U64(u64),
    Vec(Vec<Box<DetailValue>>),
    Slice(serde_bytes::ByteBuf),
    Text(String),
    True,
    False,
    Float(f64),
    #[serde(rename = "Principal")]
    Principal_(Principal),
}

candid::define_service!(pub Swap : {
  "addRecord" : candid::func!(
    (Principal, String, Vec<(String,DetailValue,)>) -> ()
  );
  "addToken" : candid::func!((Principal) -> ());
});
pub struct Service(pub Principal);
impl Service {
    pub async fn add_record(
        &self,
        arg0: Principal,
        arg1: String,
        arg2: Vec<(String, DetailValue)>,
    ) -> Result<()> {
        ic_cdk::call(self.0, "addRecord", (arg0, arg1, arg2)).await
    }
    pub async fn add_token(&self, arg0: Principal) -> Result<()> {
        ic_cdk::call(self.0, "addToken", (arg0,)).await
    }
}
