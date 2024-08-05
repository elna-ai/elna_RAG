use candid::CandidType;
mod types;
use ic_cdk::api::call::RejectionCode;
use ic_cdk::api::management_canister::http_request::HttpResponse;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use ic_cdk_macros::init;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
mod helpers;
use helpers::canister_calls::{delete_collection_from_db, get_agent_details, get_db_file_names};
use helpers::history::{History, Roles};
use helpers::out_calls::{post_json, transform_impl};
use helpers::prompt::{get_prompt, SUMMARY};
use ic_cdk::{export_candid, post_upgrade, query, update};
use std::fmt::Write;
use time::{Duration, OffsetDateTime};

pub fn fromat_time() -> String {
    let timestamp_ns = ic_cdk::api::time();
    let timestamp_s = timestamp_ns / 1_000_000_000;
    let nanos_remainder = timestamp_ns % 1_000_000_000;

    let time = OffsetDateTime::from_unix_timestamp(timestamp_s as i64).unwrap()
        + Duration::nanoseconds(nanos_remainder as i64);

    format!(
        "{:02}:{:02}:{:02} on {:02}/{:02}/{:04}",
        time.hour(),
        time.minute(),
        time.second(),
        time.day(),
        time.month() as u32,
        time.year(),
    )
}

thread_local! {
    static ENVS: RefCell<Envs> = RefCell::default();

}

#[derive(Deserialize, CandidType, Debug, Default)]
pub struct Envs {
    wizard_details_canister_id: String,
    external_service_url: String,
    vectordb_canister_id: String,
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
#[update]
async fn format_query(
    agent_id: String,
    query_text: String,
    history: Vec<(History, History)>,
) -> String {
    let history_vec = {
        if history.is_empty() {
            let caller_id = ic_cdk::caller().to_string();
            History::read_history(&caller_id, agent_id)
        } else {
            history.clone()
        }
    };

    if !history_vec.is_empty() {
        let mut formatted_query = String::from("");
        let mut previous_chats = String::from("");
        SUMMARY.with(|summary| {
            let summary = summary.borrow();

            if !summary.is_empty() {
                previous_chats = summary.clone();

                let (history1, history2) = &history_vec[history_vec.len() - 1];

                writeln!(previous_chats, "{:?}: {}", history1.role, history1.content,).unwrap();
                writeln!(previous_chats, "{:?}: {}", history2.role, history2.content,).unwrap();
            } else {
                let history_len = history_vec.len();
                let start_index = if history_len >= 2 { history_len - 2 } else { 0 };

                for i in start_index..history_len {
                    let (history1, history2) = &history_vec[i];
                    writeln!(previous_chats, "{:?}: {} ", history1.role, history1.content,)
                        .unwrap();
                    writeln!(previous_chats, "{:?}: {}", history2.role, history2.content,).unwrap();
                }
            }
        });

        writeln!(
            formatted_query,
            " previous conversation context:{} Current question:{}",
            previous_chats, query_text
        )
        .unwrap();
        ic_cdk::println!("{}", formatted_query);
        formatted_query
    } else {
        query_text
    }
}
#[update]
async fn chat(
    agent_id: String,
    query_text: String,
    query_vector: Vec<f32>,
    uuid: String,
    history: Vec<(History, History)>,
) -> Result<Response, Error> {
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
    ic_cdk::println!("Agent history: {:?}", agent_history);

    let agent = Agent {
        query_text: query_text.clone(),
        biography: wizard_details.biography,
        greeting: wizard_details.greeting,

        query_vector: query_vector,
        index_name: agent_id.clone(),
        history: agent_history,
    };

    let hist_uid = uuid.clone() + "_history";

    let message = get_prompt(agent, 2, hist_uid.to_string()).await;

    let external_url = get_envs().external_service_url;

    let response: Result<Response, Error> = post_json::<Message, Response>(
        format!("{}/canister-chat", external_url).as_str(),
        message,
        uuid.to_string().clone(),
        None,
    )
    .await;
    match response {
        Ok(data) => {
            if history.is_empty() {
                let time = fromat_time();
                let history_entry1 = History {
                    role: Roles::User,
                    content: query_text,
                    timestamp: time.clone(),
                };
                let history_entry2 = History {
                    role: Roles::Assistant,
                    content: data.body.response.clone(),
                    timestamp: time,
                };
                let history_entries = (history_entry1, history_entry2);
                History::record_history(history_entries, agent_id.clone(), &caller);
            }
            Ok(data)
        }
        Err(e) => Err(e),
    }
}

#[update]
async fn get_file_names(
    index_name: String,
) -> Result<Vec<String>, (RejectionCode, String, String)> {
    get_db_file_names(index_name).await
}

#[update]
async fn delete_collections_(index_name: String) -> Result<String, (RejectionCode, String)> {
    delete_collection_from_db(index_name).await
}

// required to process response from outbound http call
// do not delete these.
#[query]
fn transform(raw: TransformArgs) -> HttpResponse {
    transform_impl(raw)
}

export_candid!();
