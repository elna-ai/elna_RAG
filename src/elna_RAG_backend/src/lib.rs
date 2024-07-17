
use candid::CandidType;
mod types;
use ic_cdk::api::call::RejectionCode;
use ic_cdk::api::management_canister::http_request::HttpResponse;
use ic_cdk::api::management_canister::http_request::TransformArgs;
use ic_cdk_macros::init;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::cell::RefCell;
mod helpers;
use helpers::history::{Roles,History};
use helpers::canister_calls::{delete_collection_from_db, get_agent_details, get_db_file_names};
use helpers::out_calls::{post_json, transform_impl};
// use helpers::prompt::get_prompt;
use helpers::prompt::get_prompt_test;
use helpers::prompt::summarise_history;
use ic_cdk::{export_candid, post_upgrade, query, update};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    query_text: String,
    biography: String,
    greeting: String,
    // query_vector: Vec<f32>,
    index_name: String,
    history: Vec<History>,
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


// #[update]
// async fn chat(
//     agent_id: String,
//     query_text: String,
//     query_vector: Vec<f32>,
//     uuid: String, // history: Vec<History>,
// ) -> Result<Response, Error> {
//     // TODO: call vector db
//     let wizard_details = match get_agent_details(agent_id.clone()).await {
//         // TODO: change error type
//         None => return Err(Error::BodyNonSerializable),
//         // return Err("wizard details not found"),
//         Some(value) => value,
//     };

//     let agent = Agent {
//         query_text: query_text,
//         biography: wizard_details.biography,
//         greeting: wizard_details.greeting,
//         // query_vector: query_vector,
//         index_name: agent_id,
//         //TODO: add history,
//     };

//     let message = get_prompt(agent, 2).await;

//     let external_url = get_envs().external_service_url;
//     let response: Result<Response, Error> = post_json::<Message, Response>(
//         format!("{}/canister-chat", external_url).as_str(),
//         message,
//         uuid,
//         None,
//     )
//     .await;
//     match response {
//         Ok(data) => Ok(data),
//         Err(e) => Err(e),
//     }
// }

#[update]
async fn testchat(
    agent_id: String,
    query_text: String,
    // query_vector: Vec<f32>,
    uuid: i32, // history: Vec<History>,
) -> Result<Response, Error> {
    // TODO: call vector db
    // let wizard_details = match get_agent_details(agent_id.clone()).await {
    //     // TODO: change error type
    //     None => return Err(Error::BodyNonSerializable),
    //     // return Err("wizard details not found"),
    //     Some(value) => value,
    // };

        
    let caller = ic_cdk::api::caller();

    let agent_history = History::read_history(caller.to_string(),agent_id.clone());

    History::record_history(Roles::User, query_text.clone(), agent_id.clone());
    let agent = Agent {
        query_text: query_text,
        // biography: wizard_details.biography,
        // greeting: wizard_details.greeting,
        biography:String::from("A medical chat assistant"),
        greeting: String::from("Hello Good Morning!"),
        // query_vector: query_vector,
        index_name: agent_id.clone(),
        history:agent_history
    };
    let hist_uid=uuid.borrow()+100;
    let message = get_prompt_test(agent, 2,hist_uid.to_string().clone()).await;

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
            // Add the assistant message to the history
            History::record_history( Roles::Assistant, data.body.response.clone(), agent_id.clone());
            Ok(data)
        },
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

#[query]
fn history_test(agent_id:String)->Vec<History>{
    let caller = ic_cdk::api::caller();
    History::read_history(caller.to_string(),agent_id.clone())    
}


#[update]
pub async fn summarise_history_test(agent_id:String,uuid:String)->String{
    let caller = ic_cdk::api::caller();
    let agent_history=History::read_history(caller.to_string(),agent_id.clone());
    let hist=summarise_history(agent_history,uuid).await;
    hist

}




export_candid!();
