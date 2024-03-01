#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::get_envs;
use candid::{self, Principal};
use ic_cdk::api::call::RejectionCode;
use crate::types::vectordb::Service as VectordbService;
use crate::types::agent_details::{Service as AgentService,WizardDetails};


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

    let verctor_db = VectordbService(Principal::from_text(get_envs().vectordb_canister_id).unwrap());
    let result = verctor_db.query(index_name,q,limit).await;

    match result {
        Ok(response) => {
            Ok(response.0.join("\n"))
        }
        Err(err) => {
            Err(err)
        }
    }
}


