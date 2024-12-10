use candid::CandidType;
mod types;

use helpers::canister_calls::embedding_model;
use ic_cdk::api::call::RejectionCode;
use ic_cdk::api::management_canister::http_request::HttpResponse;
use ic_cdk::api::management_canister::http_request::TransformArgs;
// use ic_cdk_macros::init;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
mod helpers;
use helpers::canister_calls::get_agent_details;
use helpers::history::{History, Roles};
use helpers::out_calls::post_json;
use helpers::prompt::get_prompt;
use ic_cdk::{export_candid, post_upgrade, update};

thread_local! {
    static ENVS: RefCell<Envs> = RefCell::default();

}

#[derive(Deserialize, CandidType, Debug, Default)]
pub struct Envs {
    wizard_details_canister_id: String,
    external_service_url: String,
    vectordb_canister_id: String,
    embedding_model_canister_id: String,
}

#[ic_cdk::init]
fn init(args: Envs) {
    ENVS.with(|envs| {
        let mut envs = envs.borrow_mut();
        envs.wizard_details_canister_id = args.wizard_details_canister_id;
        envs.external_service_url = args.external_service_url;
        envs.vectordb_canister_id = args.vectordb_canister_id;
        envs.embedding_model_canister_id = args.embedding_model_canister_id;
    })
}

#[post_upgrade]
fn upgrade_env(args: Envs) {
    init(args);
}

pub fn get_envs() -> Envs {
    ENVS.with(|env| {
        let env = env.borrow();
        Envs {
            wizard_details_canister_id: env.wizard_details_canister_id.clone(),
            external_service_url: env.external_service_url.clone(),
            vectordb_canister_id: env.vectordb_canister_id.clone(),
            embedding_model_canister_id: env.embedding_model_canister_id.clone(),
        }
    })
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    query_text: String,
    biography: String,
    greeting: String,
    query_vector: Vec<f32>,
    index_name: String,
    history: Vec<(History, History)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    system_message: String,
    user_message: String,
}

#[derive(Deserialize, CandidType, Debug)]
pub struct Body {
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

#[update]
async fn chat(
    agent_id: String,
    query_text: String,
    query_vector: Option<Vec<f32>>,
    uuid: String,
    history: Vec<(History, History)>,
) -> Result<Response, Error> {
    ic_cdk::println!("Agent ID: {:?}", agent_id);
    let wizard_details = match get_agent_details(agent_id.clone()).await {
        // TODO: change error type
        None => return Err(Error::BodyNonSerializable),
        // return Err("wizard details not found"),
        Some(value) => value,
    };

    let caller = ic_cdk::api::caller().to_string();

    let agent_history: Vec<(History, History)> = if history.is_empty() {
        History::read_history(&caller, agent_id.clone())
    } else {
        history.clone()
    };
    ic_cdk::println!("Query Text: {:?}", query_text);
    ic_cdk::println!("Agent history: {:?}", agent_history);

    let vectors = match query_vector {
        Some(vector) => vector,
        None => embedding_model(query_text.clone()).await,
    };

    let agent = Agent {
        query_text: query_text.clone(),
        biography: wizard_details.biography,
        greeting: wizard_details.greeting,

        query_vector: vectors,
        index_name: agent_id.clone(),
        history: agent_history,
    };

    let hist_uid = uuid.clone() + "_history";

    let message = get_prompt(agent, 2, hist_uid.to_string()).await;

    let external_url = get_envs().external_service_url;
    ic_cdk::println!("HTTP out call");
    let response: Result<Response, Error> = post_json::<Message, Response>(
        format!("{}/canister-chat", external_url).as_str(),
        message,
        uuid.to_string().clone(),
        None,
    )
    .await;
    ic_cdk::println!("Response: {:?}", response);

    match response {
        Ok(data) => {
            // Record history if it was None initially
            if history.is_empty() {
                let history_entry1 = History {
                    role: Roles::User,
                    content: query_text,
                    // timestamp: time.clone(),
                };
                let history_entry2 = History {
                    role: Roles::Assistant,
                    content: data.body.response.clone(),
                    // timestamp: time,
                };
                let history_entries = (history_entry1, history_entry2);
                History::record_history(history_entries, agent_id.clone(), &caller);
            }
            Ok(data)
        }
        Err(e) => Err(e),
    }
}

export_candid!();
