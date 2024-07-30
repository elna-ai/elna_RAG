use serde::{Serialize};
// use time::OffsetDateTime;
// use time::format_description::well_known::Rfc3339;
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{storable,BoundedStorable,
     DefaultMemoryImpl, StableBTreeMap, Storable,
};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 = 100;





#[derive(PartialEq, Eq, PartialOrd, Ord, Clone,Debug, Serialize, Deserialize, CandidType)]
struct Caller_id(String);

impl Storable for Caller_id{
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // String already implements `Storable`.
        self.0.to_bytes()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(String::from_bytes(bytes))
    }

}

impl BoundedStorable for Caller_id {
    const MAX_SIZE: u32=MAX_VALUE_SIZE;

    const IS_FIXED_SIZE: bool=false;
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone,Debug, Serialize, Deserialize, CandidType)]
struct Agent_id(String);

impl Storable for Agent_id{
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // String already implements `Storable`.
        self.0.to_bytes()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(String::from_bytes(bytes))
    }

}

impl BoundedStorable for Agent_id {
    const MAX_SIZE: u32=MAX_VALUE_SIZE;

    const IS_FIXED_SIZE: bool=false;
}


#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]

struct Content(Vec<(History,History)>);

impl Storable for Content {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }


}

impl BoundedStorable for Content {
    const MAX_SIZE: u32=500;

    const IS_FIXED_SIZE: bool=false;
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

impl Storable for History {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

}
impl BoundedStorable for History {
    const MAX_SIZE: u32=10000;

    const IS_FIXED_SIZE: bool=false;
}

struct AgentContentMap(StableBTreeMap<Agent_id, Content, Memory>);

impl Storable for AgentContentMap {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // Serialize the inner map
        let encoded: Vec<u8> = Encode!(&self.0.iter().collect::<Vec<_>>()).unwrap();
        Cow::Owned(encoded)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        // Deserialize into a vector of key-value pairs
        let decoded: Vec<(Agent_id, Content)> = Decode!(bytes.as_ref(), Vec<(Agent_id, Content)>).unwrap();
        let mut map = StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))));
        for (k, v) in decoded {
            map.insert(k, v);
        }
        AgentContentMap(map)
    }
}

impl BoundedStorable for AgentContentMap {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE;
    const IS_FIXED_SIZE: bool = false;
}


thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static MAP: RefCell<StableBTreeMap<Caller_id, AgentContentMap, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

// thread_local! {
//     static HISTORY_MAP: RefCell<HashMap<String, HashMap<String, Vec<(History, History)>>>> = RefCell::new(HashMap::new());
// }
// impl History {
//     pub fn record_history(history_entry:(History,History),agent_id: String, caller: &String) {
//         // let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
//         // let time=now.format(&Rfc3339).unwrap();


//         HISTORY_MAP.with(|map| {
//             let mut map = map.borrow_mut();
//             map.entry(caller.clone())
//                 .or_insert_with(HashMap::new)
//                 .entry(agent_id.clone())
//                 .or_insert_with(Vec::new)
//                 .push(history_entry);
//         });
//     }

//     pub fn read_history(caller_id: &String, agent_id: String) -> Vec<(History, History)> {
//         HISTORY_MAP.with(|map| {
//             map.borrow()
//                 .get(caller_id)
//                 .and_then(|agent_map| agent_map.get(&agent_id))
//                 .cloned()
//                 .unwrap_or_else(Vec::new)
//         })
//     }
// }