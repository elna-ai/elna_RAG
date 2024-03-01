use candid::CandidType;
mod types;

use ic_cdk::api::management_canister::http_request::HttpResponse;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use ic_cdk_macros::init;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
mod helpers;
use helpers::canister_calls::get_agent_details;
use helpers::prompt::get_prompt;
use helpers::out_calls::post_json;
use helpers::out_calls::transform_impl;
use ic_cdk::{export_candid, query, update};


thread_local! {
    static ENVS: RefCell<Envs> = RefCell::default();
}
#[derive(Deserialize, CandidType, Debug, Default)]
pub struct Envs {
    wizard_details_canister_id: String,
    external_service_url: String,
    vectordb_canister_id:String
}

#[init]
fn init(args: Envs) {
    ENVS.with(|envs| {
        let mut envs = envs.borrow_mut();
        envs.wizard_details_canister_id = args.wizard_details_canister_id;
        envs.external_service_url = args.external_service_url;
        envs.vectordb_canister_id = args.vectordb_canister_id;
    })
}


pub fn get_envs() -> Envs {
    ENVS.with(|env| {
        let env = env.borrow();
        Envs {
            wizard_details_canister_id: env.wizard_details_canister_id.clone(),
            external_service_url: env.external_service_url.clone(),
            vectordb_canister_id:env.vectordb_canister_id.clone()
        }
    })
}


// TODO: make sure role can only be "user" or "assistant"?
#[derive(Debug, Serialize, Deserialize, CandidType)]
struct History {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    query_text: String,
    biography: String,
    greeting: String,
    query_vector: Vec<f32>,
    index_name: String,
    // history: Vec<History>,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct Message{
    system_message:String,
    user_message:String
}

#[derive(Deserialize, CandidType, Debug)]
struct Body {
    response: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, CandidType, Debug)]
struct Response {
    statusCode: u16,
    body: Body,
}

#[derive(CandidType, Debug)]
pub enum Error {
    ParseError,
    CantParseHost,
    HttpError(String),
    BodyNonSerializable,
}

// impl From<ParseError> for Error {
//     fn from(error: ParseError) -> Self {
//         Error::ParseErrorT::ParseError(error)
//     }
// }



#[update]
async fn chat(
    agent_id: String,
    query_text: String,
    query_vector: Vec<f32>,
    // history: Vec<History>,
) -> Result<Response, Error> {
    // TODO: call vector db
    let wizard_details = match get_agent_details(agent_id.clone()).await {
        // TODO: change error type
        None => return Err(Error::BodyNonSerializable),
        // return Err("wizard details not found"),
        Some(value) => value,
    };

    let agent = Agent {
        query_text: query_text,
        biography: wizard_details.biography,
        greeting: wizard_details.greeting,
        query_vector: query_vector,
        index_name:agent_id 
        
        // history,
    };

    let message = get_prompt(agent,2).await;


    let external_url = get_envs().external_service_url;
    let response: Result<Response, Error> = post_json::<Message, Response>(
        format!("{}/canister-chat", external_url).as_str(),
        message,
    )
    .await;
    match response {
        Ok(data) => Ok(data),
        Err(e) => Err(e),
    }
}




// required to process response from outbound http call
// do not delete these.
#[query]
fn transform(raw: TransformArgs) -> HttpResponse {
    transform_impl(raw)
}



export_candid!();
