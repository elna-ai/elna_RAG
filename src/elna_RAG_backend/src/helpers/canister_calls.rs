#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::get_envs;
use crate::types::agent_details::{ Service as AgentService, WizardDetailsV3 };
use crate::types::cap::{ DetailValue, Service as CapService };
use crate::types::embedding::Service as EmbeddingService;
use crate::types::vectordb::{ Result1, Result_, Service as VectordbService };
use candid::{ self, Principal };
use ic_cdk::api::call::RejectionCode;
use std::cell::RefCell;
use std::collections::{ HashMap, HashSet };
use ic_cdk::api::time;

// Global state to track upload sessions
thread_local! {
    static UPLOAD_SESSIONS: RefCell<HashMap<String, UploadSession>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Debug)]
struct UploadSession {
    index_name: String,
    total_files: usize,
    files_info: HashMap<String, FileInfo>,
    created_at: u64,
    collection_created: bool,
    index_built: bool,
    agent_updated: bool,
}

#[derive(Clone, Debug)]
struct FileInfo {
    total_chunks: usize,
    received_chunks: HashSet<usize>,
    completed: bool,
}

pub async fn get_agent_details(wizard_id: String) -> Option<WizardDetailsV3> {
    let canister_id = get_envs().wizard_details_canister_id;
    let wizard_details_service = AgentService(Principal::from_text(canister_id).unwrap());
    let result = wizard_details_service.get_wizard(wizard_id).await;
    match result {
        Ok((wizard_details,)) => wizard_details,
        _ => None,
    }
}

pub async fn update_agent_details(wizard_id: String) -> Result<String, (RejectionCode, String)> {
    let canister_id = get_envs().wizard_details_canister_id;
    let wizard_details_service = AgentService(Principal::from_text(canister_id).unwrap());
    let result = wizard_details_service.update_knowledge_analytics(wizard_id).await;
    match result {
        Ok((result1,)) => Ok(result1),
        Err(rejection) => Err(rejection),
    }
}

#[ic_cdk::update]
async fn create_collection(
    index_name: String,
    size: usize
) -> Result<String, (RejectionCode, String)> {
    let canister_id = get_envs().vectordb_canister_id;
    ic_cdk::println!("canister_id {:#?}", canister_id);
    let vector_db = VectordbService(Principal::from_text(canister_id).unwrap());
    ic_cdk::println!("Inserting agent {:#?}", index_name);
    let result = vector_db.create_collection(index_name.clone(), size).await;
    ic_cdk::println!("Agent {:#?} inserted", index_name);
    match result {
        Ok(result1) =>
            match result1.0 {
                Result_::Ok => { Ok("Index Created".to_string()) }
                Result_::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
            }
        Err(rejection) => Err(rejection),
    }
}

#[ic_cdk::update]
async fn insert_data(
    index_name: String,
    documents: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    file_name: String
) -> Result<String, (RejectionCode, String)> {
    let canister_id = get_envs().vectordb_canister_id;
    let vector_db = VectordbService(Principal::from_text(canister_id).unwrap());
    let vec_result = vector_db.insert(index_name, embeddings, documents, file_name).await;
    match vec_result {
        Ok(result1) =>
            match result1.0 {
                Result_::Ok => Ok("Data Inserted".to_string()),
                Result_::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
            }
        Err(rejection) => Err(rejection),
    }
}

#[ic_cdk::update]
async fn build_index(index_name: String) -> Result<String, (RejectionCode, String)> {
    let canister_id = get_envs().vectordb_canister_id;
    let vector_db = VectordbService(Principal::from_text(canister_id).unwrap());
    let result = vector_db.build_index(index_name.clone()).await;
    match result {
        Ok(result1) =>
            match result1.0 {
                Result_::Ok => {
                    // Call update_agent_details after successful index build
                    update_agent_details(index_name).await?;
                    Ok("Done".to_string())
                }
                Result_::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
            }
        Err(rejection) => Err(rejection),
    }
}

#[ic_cdk::update]
async fn insert_batch_with_state_management(
    session_id: String,
    index_name: String,
    documents: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    file_name: String,
    chunk_index: usize,
    total_chunks: usize,
    total_files: usize
) -> Result<String, (RejectionCode, String)> {
    let canister_id = get_envs().vectordb_canister_id;
    let vector_db = VectordbService(Principal::from_text(canister_id).unwrap());

    ic_cdk::println!("=== BATCH UPLOAD DEBUG ===");
    ic_cdk::println!("Session ID: {}", session_id);
    ic_cdk::println!("Index Name: {}", index_name);
    ic_cdk::println!("File Name: {}", file_name);
    ic_cdk::println!("Chunk Index: {}/{}", chunk_index + 1, total_chunks);
    ic_cdk::println!("Total Files: {}", total_files);
    ic_cdk::println!("Documents count: {}", documents.len());
    ic_cdk::println!("Embeddings count: {}", embeddings.len());

    // Validate input
    if documents.len() != embeddings.len() {
        return Err((
            RejectionCode::CanisterError,
            format!(
                "Documents count ({}) doesn't match embeddings count ({})",
                documents.len(),
                embeddings.len()
            ),
        ));
    }

    if chunk_index >= total_chunks {
        return Err((
            RejectionCode::CanisterError,
            format!("Invalid chunk_index {} for total_chunks {}", chunk_index, total_chunks),
        ));
    }

    // Get or create upload session
    let mut session_created = false;
    let mut need_to_create_collection = false;

    let (session_total_files, file_exists) = UPLOAD_SESSIONS.with(|sessions| {
        let mut sessions_map = sessions.borrow_mut();
        let session = sessions_map.entry(session_id.clone()).or_insert_with(|| {
            session_created = true;
            ic_cdk::println!("Creating new upload session: {}", session_id);
            UploadSession {
                index_name: index_name.clone(),
                total_files,
                files_info: HashMap::new(),
                created_at: time(),
                collection_created: false,
                index_built: false,
                agent_updated: false,
            }
        });

        // Validate session consistency
        if session.index_name != index_name {
            ic_cdk::println!("⚠️ Warning: Index name mismatch for session {}", session_id);
        }
        if session.total_files != total_files {
            ic_cdk::println!("⚠️ Warning: Total files mismatch for session {}", session_id);
        }

        // Check if this file already exists in the session
        let file_exists = session.files_info.contains_key(&file_name);

        // Add or update file info
        session.files_info.entry(file_name.clone()).or_insert_with(|| {
            ic_cdk::println!("Adding new file to session: {}", file_name);
            FileInfo {
                total_chunks,
                received_chunks: HashSet::new(),
                completed: false,
            }
        });

        need_to_create_collection = !session.collection_created;
        (session.total_files, file_exists)
    });

    if session_created {
        ic_cdk::println!("✨ New session created for: {}", session_id);
    } else {
        ic_cdk::println!("📋 Using existing session: {}", session_id);
    }

    if !file_exists {
        ic_cdk::println!("📄 New file added to session: {}", file_name);
    }

    // Check if collection exists and create only if it doesn't exist
    if need_to_create_collection {
        ic_cdk::println!("🔍 Checking if collection exists: {}", index_name);

        // First check if collection already exists in the vector database
        let collections_result = vector_db.get_docs(index_name.clone()).await;
        let collection_exists_in_db = match collections_result {
            Ok((collections_response,)) => {
                match collections_response {
                    Result1::Ok(_docs) => {
                        // If get_docs succeeds, collection exists (even if empty)
                        ic_cdk::println!("📦 Collection '{}' exists in DB: true", index_name);
                        true
                    }
                    Result1::Err(err) => {
                        // Collection doesn't exist or other error
                        ic_cdk::println!("📦 Collection '{}' exists in DB: false (error: {})", index_name, err);
                        false
                    }
                }
            }
            Err(rejection) => {
                ic_cdk::println!("⚠️ Warning: Failed to get docs: {:?}", rejection);
                false
            }
        };

        // Only create collection if it doesn't exist in the database
        if !collection_exists_in_db {
            ic_cdk::println!("🏗️ Creating new collection: {}", index_name);

            // Use a reasonable embedding dimension size
            let embedding_size = if !embeddings.is_empty() && !embeddings[0].is_empty() {
                embeddings[0].len()
            } else {
                1536 // Default OpenAI embedding size
            };

            ic_cdk::println!("📐 Collection embedding size: {}", embedding_size);

            let create_result = vector_db.create_collection(
                index_name.clone(),
                embedding_size
            ).await;
            match create_result {
                Ok((result1,)) =>
                    match result1 {
                        Result_::Ok => {
                            ic_cdk::println!(
                                "✅ New collection created successfully: {}",
                                index_name
                            );
                        }
                        Result_::Err(err) => {
                            ic_cdk::println!("❌ Failed to create collection: {}", err);
                            return Err((
                                RejectionCode::CanisterError,
                                format!("Failed to create collection: {}", err),
                            ));
                        }
                    }
                Err(rejection) => {
                    ic_cdk::println!("❌ Collection creation rejected: {:?}", rejection);
                    return Err(rejection);
                }
            }
        } else {
            ic_cdk::println!("📦 Collection '{}' already exists, will append to it", index_name);
        }

        // Mark collection as created for this session
        UPLOAD_SESSIONS.with(|sessions| {
            if let Some(session) = sessions.borrow_mut().get_mut(&session_id) {
                session.collection_created = true;
            }
        });
    } else {
        ic_cdk::println!("📦 Collection handling already completed for session: {}", session_id);
    }

    // Insert current chunk
    ic_cdk::println!(
        "📤 Inserting chunk {}/{} for file {} in session {}",
        chunk_index + 1,
        total_chunks,
        file_name,
        session_id
    );
    ic_cdk::println!(
        "📊 Chunk data - Documents: {}, Embeddings: {}",
        documents.len(),
        embeddings.len()
    );

    let insert_result = vector_db.insert(
        index_name.clone(),
        embeddings,
        documents,
        file_name.clone()
    ).await;

    match insert_result {
        Ok((result1,)) =>
            match result1 {
                Result_::Ok => {
                    ic_cdk::println!(
                        "✅ Chunk {} inserted successfully for file {} in session {}",
                        chunk_index + 1,
                        file_name,
                        session_id
                    );
                }
                Result_::Err(err) => {
                    ic_cdk::println!("❌ Failed to insert chunk {}: {}", chunk_index + 1, err);
                    return Err((
                        RejectionCode::CanisterError,
                        format!("Failed to insert chunk: {}", err),
                    ));
                }
            }
        Err(rejection) => {
            ic_cdk::println!("❌ Chunk insertion rejected: {:?}", rejection);
            return Err(rejection);
        }
    }

    // Update session state with received chunk and check completion
    let (file_completed, all_files_completed) = UPLOAD_SESSIONS.with(|sessions| {
        let mut sessions_map = sessions.borrow_mut();
        if let Some(session) = sessions_map.get_mut(&session_id) {
            if let Some(file_info) = session.files_info.get_mut(&file_name) {
                file_info.received_chunks.insert(chunk_index);
                
                // Check if this file is complete and capture data we need
                let file_complete = file_info.received_chunks.len() == file_info.total_chunks;
                let current_chunks = file_info.received_chunks.len();
                let total_chunks_for_file = file_info.total_chunks;
                
                if file_complete && !file_info.completed {
                    file_info.completed = true;
                    ic_cdk::println!("🎯 File '{}' completed for session {}", file_name, session_id);
                }
                
                // Calculate completion status after all mutable operations are done
                // Collect all the data we need without keeping any borrows
                let files_completion_data: Vec<(bool, usize, usize)> = session.files_info
                    .values()
                    .map(|info| (info.completed, info.received_chunks.len(), info.total_chunks))
                    .collect();
                
                let completed_files = files_completion_data.iter().filter(|(completed, _, _)| *completed).count();
                let all_complete = completed_files == session.total_files;
                let total_files_in_session = session.total_files;
                
                // Now we can safely print without any active borrows
                ic_cdk::println!("📈 Session {} progress: {}/{} files completed", 
                    session_id, completed_files, total_files_in_session);
                ic_cdk::println!("📄 File '{}' chunks: {}/{}", 
                    file_name, current_chunks, total_chunks_for_file);
                
                (file_complete, all_complete)
            } else {
                ic_cdk::println!("⚠️ File info not found for {} in session {}", file_name, session_id);
                (false, false)
            }
        } else {
            ic_cdk::println!("⚠️ Session {} not found when updating chunk status", session_id);
            (false, false)
        }
    });

    // Build index only after all files and chunks are received
    if all_files_completed {
        ic_cdk::println!("🎯 All files completed for session {}. Building index...", session_id);

        let build_result = vector_db.build_index(index_name.clone()).await;
        match build_result {
            Ok((result1,)) =>
                match result1 {
                    Result_::Ok => {
                        ic_cdk::println!("✅ Index built successfully for session {}", session_id);

                        // Mark index as built
                        UPLOAD_SESSIONS.with(|sessions| {
                            if let Some(session) = sessions.borrow_mut().get_mut(&session_id) {
                                session.index_built = true;
                            }
                        });

                        // Update agent details after successful index build
                        let update_result = update_agent_details(index_name.clone()).await;
                        match update_result {
                            Ok(result) => {
                                ic_cdk::println!(
                                    "✅ Agent details updated successfully: {}",
                                    result
                                );

                                // Mark agent as updated
                                UPLOAD_SESSIONS.with(|sessions| {
                                    if
                                        let Some(session) = sessions
                                            .borrow_mut()
                                            .get_mut(&session_id)
                                    {
                                        session.agent_updated = true;
                                    }
                                });
                            }
                            Err(err) => {
                                ic_cdk::println!(
                                    "⚠️ Warning: Failed to update agent details: {:?}",
                                    err
                                );
                            }
                        }

                        // Clean up session
                        UPLOAD_SESSIONS.with(|sessions| {
                            sessions.borrow_mut().remove(&session_id);
                        });

                        ic_cdk::println!(
                            "🎉 Upload session {} completed successfully with {} files!",
                            session_id,
                            session_total_files
                        );
                        return Ok(
                            format!(
                                "Upload session {} completed successfully. {} files processed, index built and agent updated.",
                                session_id,
                                session_total_files
                            )
                        );
                    }
                    Result_::Err(err) => {
                        ic_cdk::println!("❌ Failed to build index: {}", err);
                        return Err((
                            RejectionCode::CanisterError,
                            format!("Failed to build index: {}", err),
                        ));
                    }
                }
            Err(rejection) => {
                ic_cdk::println!("❌ Index build rejected: {:?}", rejection);
                return Err(rejection);
            }
        }
    }

    let status_msg = if file_completed {
        format!(
            "File '{}' completed ({}/{} chunks) for session {}",
            file_name,
            total_chunks,
            total_chunks,
            session_id
        )
    } else {
        format!(
            "Chunk {}/{} processed for file '{}' in session {}",
            chunk_index + 1,
            total_chunks,
            file_name,
            session_id
        )
    };

    ic_cdk::println!("✅ {}", status_msg);
    Ok(status_msg)
}

// Updated session status function
#[ic_cdk::query]
fn get_session_status(session_id: String) -> Option<String> {
    UPLOAD_SESSIONS.with(|sessions| {
        let sessions_map = sessions.borrow();
        if let Some(session) = sessions_map.get(&session_id) {
            let completed_files = session.files_info
                .values()
                .filter(|info| info.completed)
                .count();
            let file_details: Vec<String> = session.files_info
                .iter()
                .map(|(name, info)|
                    format!("{}({}/{})", name, info.received_chunks.len(), info.total_chunks)
                )
                .collect();

            Some(
                format!(
                    "Session: {} | Index: {} | Files: {}/{} completed | Collection: {} | Index Built: {} | Agent Updated: {} | Files: [{}]",
                    session_id,
                    session.index_name,
                    completed_files,
                    session.total_files,
                    session.collection_created,
                    session.index_built,
                    session.agent_updated,
                    file_details.join(", ")
                )
            )
        } else {
            None
        }
    })
}

#[ic_cdk::query]
fn list_active_sessions() -> Vec<String> {
    UPLOAD_SESSIONS.with(|sessions| {
        let sessions_map = sessions.borrow();
        sessions_map.keys().cloned().collect()
    })
}

// Cleanup function to remove stale sessions (call periodically)
#[ic_cdk::update]
fn cleanup_stale_sessions(max_age_seconds: u64) -> usize {
    let current_time = time();
    let mut removed_count = 0;

    UPLOAD_SESSIONS.with(|sessions| {
        let mut sessions_map = sessions.borrow_mut();
        let initial_count = sessions_map.len();

        sessions_map.retain(|session_id, session| {
            let age = current_time - session.created_at;
            let is_fresh = age < max_age_seconds * 1_000_000_000; // Convert to nanoseconds
            if !is_fresh {
                ic_cdk::println!(
                    "🧹 Removing stale session: {} (age: {}s)",
                    session_id,
                    age / 1_000_000_000
                );
            }
            is_fresh
        });

        removed_count = initial_count - sessions_map.len();
    });

    ic_cdk::println!("🧹 Cleanup completed. Removed {} stale sessions", removed_count);
    removed_count
}

// Force cleanup a specific session
#[ic_cdk::update]
fn force_cleanup_session(session_id: String) -> bool {
    UPLOAD_SESSIONS.with(|sessions| {
        let removed = sessions.borrow_mut().remove(&session_id).is_some();
        if removed {
            ic_cdk::println!("🗑️ Force removed session: {}", session_id);
        } else {
            ic_cdk::println!("⚠️ Session not found for force cleanup: {}", session_id);
        }
        removed
    })
}

#[ic_cdk::update]
async fn create_index(
    index_name: String,
    size: usize,
    documents: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    file_name: String
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
    let result = update_agent_details(index_name).await?;
    ic_cdk::println!("Updating Wizard{}", result);
    Ok("Index created successfully".to_string())
}

#[ic_cdk::update]
pub async fn search(
    index_name: String,
    embeddings: Vec<f32>,
    limit: i32
) -> Result<String, (RejectionCode, String)> {
    let vector_db = VectordbService(Principal::from_text(get_envs().vectordb_canister_id).unwrap());
    let result = vector_db.query(index_name, embeddings, limit).await;
    match result {
        Ok(response) =>
            match response.0 {
                Result1::Ok(results) => Ok(results.join("\n")),
                Result1::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
            }
        Err(err) => Err(err),
    }
}

#[ic_cdk::update]
async fn get_db_file_names(
    index_name: String
) -> Result<Vec<String>, (RejectionCode, String, String)> {
    ic_cdk::println!("Fetch DB filenames ");
    let caller = ic_cdk::api::caller();
    let vector_db = VectordbService(Principal::from_text(get_envs().vectordb_canister_id).unwrap());
    let result = vector_db.get_docs(index_name).await;
    match result {
        Ok((result2,)) =>
            match result2 {
                Result1::Ok(vec) => Ok(vec),
                Result1::Err(err) =>
                    Err((RejectionCode::CanisterError, err.to_string(), caller.to_string())),
            }
        Err(rejection) => Err((rejection.0, rejection.1, caller.to_string())),
    }
}

#[ic_cdk::update]
async fn delete_collection_from_db(index_name: String) -> Result<String, (RejectionCode, String)> {
    let vector_db = VectordbService(Principal::from_text(get_envs().vectordb_canister_id).unwrap());
    let result = vector_db.delete_collection(index_name).await;
    match result {
        Ok((result1,)) =>
            match result1 {
                Result_::Ok => Ok("Successfully Deleted".to_string()),
                Result_::Err(err) => Err((RejectionCode::CanisterError, err.to_string())),
            }
        Err(rejection) => Err(rejection),
    }
}

#[ic_cdk::update]
pub async fn embedding_model(text: String) -> Vec<f32> {
    let canister_id = get_envs().embedding_model_canister_id;
    let embedding_service = EmbeddingService(Principal::from_text(canister_id).unwrap());
    let result: Result<(Vec<f32>,), (RejectionCode, String)> = embedding_service.get_embeddings(
        text
    ).await;
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
    details: Vec<(String, DetailValue)>
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
    let val: Vec<(String, DetailValue)> = vec![(
        "test".to_string(),
        DetailValue::Text("test".to_string()),
    )];
    let cap = CapService(cid).add_record(caller_id, agent_id, val).await;
    cap
}
