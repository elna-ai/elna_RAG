use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use serde::Serialize;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 = 100;
const MAX_LARGE_SIZE: u32 = 10_485_760;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static MAP: RefCell<StableBTreeMap<CallerId, AgentContentMap, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize, CandidType)]
struct CallerId(String);

impl Storable for CallerId {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // String already implements `Storable`.
        self.0.to_bytes()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(String::from_bytes(bytes))
    }
}

impl BoundedStorable for CallerId {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE;

    const IS_FIXED_SIZE: bool = false;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize, CandidType)]
struct AgentId(String);

impl Storable for AgentId {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // String already implements `Storable`.
        self.0.to_bytes()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(String::from_bytes(bytes))
    }
}

impl BoundedStorable for AgentId {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE;

    const IS_FIXED_SIZE: bool = false;
}

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]

struct Content(Vec<(History, History)>);

impl Storable for Content {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Content {
    const MAX_SIZE: u32 = MAX_LARGE_SIZE;

    const IS_FIXED_SIZE: bool = false;
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
    pub timestamp: String,
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
    const MAX_SIZE: u32 = MAX_LARGE_SIZE;

    const IS_FIXED_SIZE: bool = false;
}

struct AgentContentMap(StableBTreeMap<AgentId, Content, Memory>);

impl Storable for AgentContentMap {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // Serialize the inner map
        let encoded: Vec<u8> = Encode!(&self.0.iter().collect::<Vec<_>>()).unwrap();
        Cow::Owned(encoded)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        // Deserialize into a vector of key-value pairs
        let decoded: Vec<(AgentId, Content)> =
            Decode!(bytes.as_ref(), Vec<(AgentId, Content)>).unwrap();
        let mut map =
            StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))));
        for (k, v) in decoded {
            map.insert(k, v);
        }
        AgentContentMap(map)
    }
}

impl BoundedStorable for AgentContentMap {
    const MAX_SIZE: u32 = MAX_LARGE_SIZE;
    const IS_FIXED_SIZE: bool = false;
}

impl History {
    pub fn record_history(history_entry: (History, History), agent_id: String, caller: &String) {
        let caller_id = CallerId(caller.clone());
        let agent_id = AgentId(agent_id);

        MAP.with(|map| {
            let mut map = map.borrow_mut();

            // Retrieve the existing AgentContentMap or create a new one if it doesn't exist
            let mut agent_map = map.get(&caller_id).unwrap_or_else(|| {
                AgentContentMap(StableBTreeMap::init(
                    MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
                ))
            });

            // Insert the history entry
            let mut content = agent_map
                .0
                .get(&agent_id)
                .unwrap_or_else(|| Content(Vec::new()));
            content.0.push(history_entry);
            agent_map.0.insert(agent_id, content.clone());

            // Update the map with the modified AgentContentMap
            map.insert(caller_id, agent_map);
        });
    }

    pub fn read_history(caller_id: &String, agent_id: String) -> Vec<(History, History)> {
        let caller_id = CallerId(caller_id.clone());
        let agent_id = AgentId(agent_id);

        MAP.with(|map| {
            let map = map.borrow();
            map.get(&caller_id)
                .and_then(|agent_map| agent_map.0.get(&agent_id).map(|content| content.clone()))
                .map(|content| content.0)
                .unwrap_or_else(Vec::new)
        })
    }
}
