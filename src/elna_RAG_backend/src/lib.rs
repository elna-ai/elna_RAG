use candid::CandidType;
use ic_cdk::api::management_canister::http_request::HttpResponse;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use serde::{Deserialize, Serialize};

mod helpers;

use candid::Principal;
use helpers::canister_calls::get_agent_details;
use helpers::out_calls::post_json;
use helpers::out_calls::transform_impl;
use ic_cdk::api::call::CallResult;
use ic_cdk::api::call::RejectionCode;
use ic_cdk::{export_candid, query, update};

type QueryResult = String;
type AgentDetails = (String, String);

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

#[derive(Debug, Serialize, Deserialize)]
struct ChatEndpoint {
    input_prompt: String,
    biography: String,
    greeting: String,
    embedding: Vec<f32>,
    // index_name: String,
    // history: Vec<History>,
}

#[derive(Deserialize, CandidType, Debug)]
struct Body {
    response: String,
}

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
    embedding: Vec<f32>,
    // history: Vec<History>,
) -> Result<Response, Error> {
    // TODO: url to env
    // TODO: call vector db
    let wizard_details = match get_agent_details(agent_id).await {
        None => return Err("wizard details not found"),
        Some(value) => value,
    };

    // TODO: need query text vector
    let data = ChatEndpoint {
        // index_name: agent_id,
        input_prompt: query_text,
        biography: wizard_details.biography,
        greeting: wizard_details.greeting,
        embedding: embedding,
        // history,
    };
    // TODO: get biography and greeting and query_text -> convert to prompt
    // TODO: url to env
    let response = post_json::<ChatEndpoint, Response>(
        "https://d2lau6bs1ulmoj.cloudfront.net/canister-chat",
        data,
    )
    .await;
    ic_cdk::api::print(format!("data received {:?}", response));
    match response {
        Ok(data) => {
            ic_cdk::api::print(format!("{:?}\n\n\n", data));
            Ok(data)
        }
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
