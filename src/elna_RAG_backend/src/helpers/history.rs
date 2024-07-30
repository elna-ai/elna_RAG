use std::collections::HashMap;
use std::cell::RefCell;
use serde::{Serialize, Deserialize};
use candid::CandidType;
// use time::OffsetDateTime;
// use time::format_description::well_known::Rfc3339;

thread_local! {
    static HISTORY_MAP: RefCell<HashMap<String, HashMap<String, Vec<(History, History)>>>> = RefCell::new(HashMap::new());
}

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub enum Roles {
    System,
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct History {
    pub role: Roles,
    pub content: String,
    // timestamp: String,
}

impl History {
    pub fn record_history(history_entry:(History,History),agent_id: String, caller: &String) {
        // let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        // let time=now.format(&Rfc3339).unwrap();


        HISTORY_MAP.with(|map| {
            let mut map = map.borrow_mut();
            map.entry(caller.clone())
                .or_insert_with(HashMap::new)
                .entry(agent_id.clone())
                .or_insert_with(Vec::new)
                .push(history_entry);
        });
    }

    pub fn read_history(caller_id: &String, agent_id: String) -> Vec<(History, History)> {
        HISTORY_MAP.with(|map| {
            map.borrow()
                .get(caller_id)
                .and_then(|agent_map| agent_map.get(&agent_id))
                .cloned()
                .unwrap_or_else(Vec::new)
        })
    }
}
