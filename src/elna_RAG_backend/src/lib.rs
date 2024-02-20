
use ic_cdk::{update,export_candid};
use candid::Principal;
use ic_cdk::api::call::CallResult;

type FooResult = String;


#[update]
async fn make_inter_canister_call()-> CallResult<(FooResult,)>{
    let word: String = "apple".into();
    let result:  CallResult<(FooResult,)> = ic_cdk::call(Principal::from_text("br5f7-7uaaa-aaaaa-qaaca-cai").unwrap(), "greet", (word,)).await;
    result

}

export_candid!();

