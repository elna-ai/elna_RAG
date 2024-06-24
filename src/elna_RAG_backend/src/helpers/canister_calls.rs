#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::get_envs;
use crate::types::agent_details::{Service as AgentService, WizardDetails};
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

pub async fn db_query(
    index_name: String,
    q: Vec<f32>,
    limit: i32,
) -> Result<String, (RejectionCode, String)> {
    let vector_db = VectordbService(Principal::from_text(get_envs().vectordb_canister_id).unwrap());
    let result = vector_db.query(index_name, q, limit).await;

    match result {
        Ok(response) => match response.0 {
            Result1::Ok(results) => Ok(results.join("\n")),
            Result1::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
        },
        Err(err) => Err(err),
    }
}

pub async fn get_db_file_names(
    index_name: String,
) -> Result<Vec<String>, (RejectionCode, String, String)> {
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

pub async fn delete_collection_from_db(
    index_name: String,
) -> Result<String, (RejectionCode, String)> {
    let vector_db = VectordbService(Principal::from_text(get_envs().vectordb_canister_id).unwrap());
    let result = vector_db.delete_collection(index_name).await;

    match result {
        Ok((result1,)) => match result1 {
            Result_::Ok => Ok("succesfully deleted".to_string()),
            Result_::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
        },
        Err(rejection) => Err(rejection),
    }
}
