use candid::CandidType;
use ic_cdk::api::management_canister::http_request::HttpResponse;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use ic_cdk_macros::init;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
mod helpers;
use candid::Principal;
use helpers::canister_calls::get_agent_details;
use helpers::out_calls::post_json;
use helpers::out_calls::transform_impl;
use ic_cdk::api::call::CallResult;
use ic_cdk::api::call::RejectionCode;
use ic_cdk::{export_candid, query, update};

type QueryResult = String;

thread_local! {
    static ENVS: RefCell<Envs> = RefCell::default();
}
#[derive(Deserialize, CandidType, Debug, Default)]
pub struct Envs {
    wizard_details_canister_id: String,
    external_service_url: String,
}

#[init]
fn init(args: Envs) {
    ENVS.with(|envs| {
        let mut envs = envs.borrow_mut();
        envs.wizard_details_canister_id = args.wizard_details_canister_id;
        envs.external_service_url = args.external_service_url;
    })
}

pub fn get_envs() -> Envs {
    ENVS.with(|env| {
        let env = env.borrow();
        Envs {
            wizard_details_canister_id: env.wizard_details_canister_id.clone(),
            external_service_url: env.external_service_url.clone(),
        }
    })
}

#[update]
async fn query(
    index_name: String,
    q: Vec<f32>,
    limit: i32,
) -> Result<String, (RejectionCode, String)> {
    // let word: String = "apple".to_string();
    let result: CallResult<(QueryResult,)> = ic_cdk::call(
        Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap(),
        "query",
        (index_name, q, limit),
    )
    .await;
    // result

    // Handle the result
    match result {
        Ok(response) => {
            // println!("Inter-canister call successful: {}", response);
            Ok(format!("{}", response.0))
        }
        Err(err) => {
            // eprintln!("Error making inter-canister call: {}", err);
            Err(err)
        }
    }
}

// TODO: make sure role can only be "user" or "assistant"?
#[derive(Debug, Serialize, Deserialize, CandidType)]
struct History {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct ChatEndpoint {
    input_prompt: String,
    biography: String,
    greeting: String,
    embedding: Vec<f32>,
    // index_name: String,
    // history: Vec<History>,
}

#[derive(Deserialize, CandidType)]
struct Body {
    response: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, CandidType)]
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
    embedding: Vec<f32>,
    // history: Vec<History>,
) -> Result<Response, Error> {
    // TODO: call vector db
    let wizard_details = match get_agent_details(agent_id).await {
        // TODO: change error type
        None => return Err(Error::BodyNonSerializable),
        // return Err("wizard details not found"),
        Some(value) => value,
    };

    let data = ChatEndpoint {
        input_prompt: query_text,
        biography: wizard_details.biography,
        greeting: wizard_details.greeting,
        embedding: embedding,
        // history,
    };
    let external_url = get_envs().external_service_url;
    let response: Result<Response, Error> = post_json::<ChatEndpoint, Response>(
        format!("{}/canister-chat", external_url).as_str(),
        data,
        None,
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
