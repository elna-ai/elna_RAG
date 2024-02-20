
use ic_cdk::api::call::RejectionCode;
use ic_cdk::{export_candid, query, update};
use candid::Principal;
use ic_cdk::api::call::CallResult;

type GreetResult = String;
type QueryResult = String;



#[update]
async fn make_inter_canister_call(name:String)-> CallResult<(GreetResult,)>{
    // let word: String = "apple".to_string();
    let result:  CallResult<(GreetResult,)> = ic_cdk::call(Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap(), "greet", (name,)).await;
    result
}


#[update]
async fn query(index_name: String, q: Vec<f32>, limit: i32)->Result<String,(RejectionCode,String)>{
    // let word: String = "apple".to_string();
    let result:  CallResult<(QueryResult,)> = ic_cdk::call(Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap(), "query", (index_name,q,limit)).await;
    // result

    // Handle the result
    match result {
        Ok(response) => {
            // println!("Inter-canister call successful: {}", response);
            Ok(format!("{}",response.0))
        }
        Err(err) => {
            // eprintln!("Error making inter-canister call: {}", err);
            Err(err)
        }
    }
    
}



export_candid!();

