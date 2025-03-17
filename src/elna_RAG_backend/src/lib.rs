use candid::CandidType;
mod types;

use candid::Principal;
use helpers::canister_calls::embedding_model;
use ic_cdk::api::call::RejectionCode;
use ic_cdk::api::management_canister::http_request::HttpResponse;
use ic_cdk::api::management_canister::http_request::TransformArgs;
// use ic_cdk_macros::init;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use types::cap::DetailValue;
mod helpers;
use helpers::canister_calls::{get_agent_details, log};
use helpers::history::{History, Roles};
use helpers::out_calls::post_json;
use helpers::prompt::get_prompt;
use ic_cdk::api::performance_counter;
use ic_cdk::{export_candid, post_upgrade, query, update};

thread_local! {
    static ENVS: RefCell<Envs> = RefCell::default();
    static LOGS: RefCell<Vec<LogEntry>> = RefCell::default();

}

#[derive(Deserialize, CandidType, Debug, Clone)]
pub struct LogEntry {
    timestamp: u64,
    caller: Principal,
    function_name: String,
    total_cycles: u64,
}

//  implement Default for LogEntry
impl Default for LogEntry {
    fn default() -> Self {
        LogEntry {
            timestamp: 0,
            caller: Principal::anonymous(), // Use a default Principal (e.g., anonymous)
            function_name: String::new(),
            total_cycles: 0,
        }
    }
}

#[derive(Deserialize, CandidType, Debug, Default)]
pub struct Envs {
    wizard_details_canister_id: String,
    external_service_url: String,
    vectordb_canister_id: String,
    embedding_model_canister_id: String,
    cap_canister_id: String,
}

#[ic_cdk::init]
fn init(args: Envs) {
    ENVS.with(|envs| {
        let mut envs = envs.borrow_mut();
        envs.wizard_details_canister_id = args.wizard_details_canister_id;
        envs.external_service_url = args.external_service_url;
        envs.vectordb_canister_id = args.vectordb_canister_id;
        envs.embedding_model_canister_id = args.embedding_model_canister_id;
        envs.cap_canister_id = args.cap_canister_id;
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
            cap_canister_id: env.cap_canister_id.clone(),
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
pub async fn delete_history(agent_id: String) -> () {
    let caller_id = ic_cdk::api::caller();
    History::clear_history(&caller_id.to_string(), agent_id.clone());
    let details: Vec<(String, DetailValue)> =
        vec![("agent_id".to_string(), DetailValue::Text(agent_id))];
    let result = log(caller_id, "delete_history".to_string(), details).await;
    ic_cdk::println!("Log : {:?}", result);
}

#[query]
pub fn get_history(agent_id: String) -> Result<Vec<(History, History)>, Error> {
    let caller = ic_cdk::api::caller();
    if caller == Principal::anonymous() {
        return Err(Error::HttpError("Anonymous user not allowed".to_string()));
    }
    Ok(History::read_history(&caller.to_string(), agent_id))
}

#[update]
async fn chat(
    agent_id: String,
    query_text: String,
    query_vector: Option<Vec<f32>>,
    uuid: String,
) -> Result<Response, Error> {
    let initial_cycles = performance_counter(0);

    let caller = ic_cdk::api::caller();
    ic_cdk::println!("Caller: {:?}", caller.to_string());

    if caller == Principal::anonymous() {
        return Err(Error::HttpError("Anonymous user not allowed".to_string()));
    }

    ic_cdk::println!("Agent ID: {:?}", agent_id);
    let wizard_details = match get_agent_details(agent_id.clone()).await {
        // TODO: change error type
        None => return Err(Error::BodyNonSerializable),
        // return Err("wizard details not found"),
        Some(value) => value,
    };

    // let mut anonymous = true;
    // let agent_history = if caller.to_string() == Principal::anonymous().to_text() {
    //     history
    // } else {
    //     anonymous = false;
    //     History::read_history(&caller.to_string(), agent_id.clone())
    // };

    let agent_history = History::read_history(&caller.to_string(), agent_id.clone());

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
            //Cap canister Logging

            let details: Vec<(String, DetailValue)> = vec![
                ("agentId".to_string(), DetailValue::Text(agent_id.clone())),
                (
                    "queryText".to_string(),
                    DetailValue::Text(query_text.clone()),
                ),
                (
                    "response".to_string(),
                    DetailValue::Text(data.body.response.clone()),
                ),
            ];
            let result = log(caller, "chat".to_string(), details).await;
            ic_cdk::println!("Log : {:?}", result);

            // Record history if it was None initially
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
            History::record_history(history_entries, agent_id.clone(), &caller.to_string());

            // Get the final cycle count

            let final_cycles = performance_counter(0);

            // Calculate the cycles used

            let cycles_used = final_cycles - initial_cycles;

            // Create a new log entry
            let log_entry = LogEntry {
                timestamp: ic_cdk::api::time(), // Use the current timestamp
                caller,
                function_name: "chat".to_string(),
                total_cycles: cycles_used,
            };

            // Append the log entry to the LOGS
            LOGS.with(|logs| {
                logs.borrow_mut().push(log_entry);
            });

            Ok(data)
        }
        Err(e) => Err(e),
    }
}
#[query]
fn get_logs() -> Vec<LogEntry> {
    LOGS.with(|logs| {
        logs.borrow().clone() // Return a clone of the logs vector
    })
}
export_candid!();
