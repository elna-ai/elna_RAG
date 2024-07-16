use std::collections::HashMap;

thread_local! {
    static HISTORY_MAP: RefCell<HashMap<String, HashMap<String, Vec<History>>>> = RefCell::new(HashMap::new());

}

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub enum Roles {
    System,
    User,
    Assistant,
}
#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct History {
    role: Roles,
    content: String,
    timestamp:String,

}

impl History {
    pub fn record_history(role: Roles, content: String, agent_id: String) {
        let history_entry = History {
            role,
            content,
            timestamp: Utc::now().to_string(),
    
        };
        let caller = ic_cdk::api::caller();
    
        HISTORY_MAP.with(|map| {
            let mut map = map.borrow_mut();
            map.entry(caller.clone().to_string())
                .or_insert_with(HashMap::new)
                .entry(agent_id.clone())
                .or_insert_with(Vec::new)
                .push(history_entry);
        });}
    

    pub fn read_history(caller_id: String, agent_id: String) -> Vec<History> {
        HISTORY_MAP.with(|map| {
            map.borrow()
                .get(&caller_id)
                .and_then(|agent_map| agent_map.get(&agent_id))
                .cloned()
                .unwrap_or_else(Vec::new)
        })
    
}
}

