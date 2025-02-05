#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::get_envs;
use crate::types::agent_details::{Service as AgentService, WizardDetails};
use crate::types::cap::{DetailValue, Service as CapService};
use crate::types::embedding::Service as EmbeddingService;
use crate::types::vectordb::{Result1, Result_, Service as VectordbService};
use candid::{self, Principal};
use ic_cdk::api::call::RejectionCode;

pub async fn get_agent_details(wizard_id: String) -> Option<WizardDetails> {
    let canister_id = get_envs().wizard_details_canister_id;
    let wizard_details_service = AgentService(Principal::from_text(canister_id).unwrap());
    let result = wizard_details_service.get_wizard(wizard_id).await;
    match result {
        Ok((wizard_details,)) => wizard_details,
        _ => None,
    }
}

#[ic_cdk::update]
async fn create_collection(
    index_name: String,
    size: usize,
) -> Result<String, (RejectionCode, String)> {
    let canister_id = get_envs().vectordb_canister_id;
    let vector_db = VectordbService(Principal::from_text(canister_id).unwrap());
    let result = vector_db.create_collection(index_name, size).await;

    match result {
        Ok(result1) => match result1.0 {
            Result_::Ok => Ok("Index Created".to_string()),
            Result_::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
        },
        Err(rejection) => Err(rejection),
    }
}

#[ic_cdk::update]
async fn insert_data(
    index_name: String,
    documents: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    file_name: String,
) -> Result<String, (RejectionCode, String)> {
    let canister_id = get_envs().vectordb_canister_id;
    let vector_db = VectordbService(Principal::from_text(canister_id).unwrap());
    let vec_result = vector_db
        .insert(index_name, embeddings, documents, file_name)
        .await;
    match vec_result {
        Ok(result1) => match result1.0 {
            Result_::Ok => Ok("Data Inserted".to_string()),
            Result_::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
        },
        Err(rejection) => Err(rejection),
    }
}

#[ic_cdk::update]
async fn build_index(index_name: String) -> Result<String, (RejectionCode, String)> {
    let canister_id = get_envs().vectordb_canister_id;
    let vector_db = VectordbService(Principal::from_text(canister_id).unwrap());
    let result = vector_db.build_index(index_name).await;
    match result {
        Ok(result1) => match result1.0 {
            Result_::Ok => Ok("Done".to_string()),
            Result_::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
        },
        Err(rejection) => Err(rejection),
    }
}

#[ic_cdk::update]
async fn create_index(
    index_name: String,
    size: usize,
    documents: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    file_name: String,
) -> Result<String, (RejectionCode, String)> {
    ic_cdk::println!("Creating Index");
    create_collection(index_name.clone(), size).await?;
    ic_cdk::println!("Collection Created");
    ic_cdk::println!("*******************");

    ic_cdk::println!("Inserting Data");
    insert_data(index_name.clone(), documents, embeddings, file_name).await?;
    ic_cdk::println!("Data Inserted");
    ic_cdk::println!("*************");

    ic_cdk::println!("Indexing..");
    build_index(index_name.clone()).await?;
    ic_cdk::println!("Index created");

    Ok("Index created successfully".to_string())
}

#[ic_cdk::update]
pub async fn search(
    index_name: String,
    embeddings: Vec<f32>,
    limit: i32,
) -> Result<String, (RejectionCode, String)> {
    let vector_db = VectordbService(Principal::from_text(get_envs().vectordb_canister_id).unwrap());
    let result = vector_db.query(index_name, embeddings, limit).await;

    match result {
        Ok(response) => match response.0 {
            Result1::Ok(results) => Ok(results.join("\n")),
            Result1::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
        },
        Err(err) => Err(err),
    }
}

#[ic_cdk::update]
async fn get_db_file_names(
    index_name: String,
) -> Result<Vec<String>, (RejectionCode, String, String)> {
    ic_cdk::println!("Fetch DB filenames ");
    let caller = ic_cdk::api::caller();
    let vector_db = VectordbService(Principal::from_text(get_envs().vectordb_canister_id).unwrap());
    let result = vector_db.get_docs(index_name).await;

    match result {
        Ok((result2,)) => match result2 {
            Result1::Ok(vec) => Ok(vec),
            Result1::Err(err) => Err((
                RejectionCode::CanisterError,
                err.to_string(),
                caller.to_string(),
            )),
        },
        Err(rejection) => Err((rejection.0, rejection.1, caller.to_string())),
    }
}

#[ic_cdk::update]
async fn delete_collection_from_db(index_name: String) -> Result<String, (RejectionCode, String)> {
    let vector_db = VectordbService(Principal::from_text(get_envs().vectordb_canister_id).unwrap());
    let result = vector_db.delete_collection(index_name).await;

    match result {
        Ok((result1,)) => match result1 {
            Result_::Ok => Ok("Successfully Deleted".to_string()),
            Result_::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
        },
        Err(rejection) => Err(rejection),
    }
}

#[ic_cdk::update]
pub async fn embedding_model(text: String) -> Vec<f32> {
    let canister_id = get_envs().embedding_model_canister_id;
    let embedding_service = EmbeddingService(Principal::from_text(canister_id).unwrap());
    let result: Result<(Vec<f32>,), (RejectionCode, String)> =
        embedding_service.get_embeddings(text).await;

    match result {
        Ok(result) => result.0,
        Err(rejection) => {
            ic_cdk::println!("Error in embedding model {:?}", rejection);
            Vec::new() // Return an empty Vec<f32> as a fallback
        }
    }
}

#[ic_cdk::update]
pub async fn log(
    caller: Principal,
    operation: String,
    details: Vec<(String, DetailValue)>,
) -> Result<(), (RejectionCode, std::string::String)> {
    let canister_id = get_envs().cap_canister_id;
    let cid = Principal::from_text(canister_id).unwrap();
    let cap = CapService(cid).add_record(caller, operation, details).await;

    cap
}

#[ic_cdk::update]
async fn test(agent_id: String) -> Result<(), (RejectionCode, std::string::String)> {
    let caller_id = ic_cdk::api::caller();
    let canister_id = get_envs().cap_canister_id;
    let cid = Principal::from_text(canister_id).unwrap();
    let val: Vec<(String, DetailValue)> =
        vec![("test".to_string(), DetailValue::Text("test".to_string()))];
    let cap = CapService(cid).add_record(caller_id, agent_id, val).await;

    cap
}
