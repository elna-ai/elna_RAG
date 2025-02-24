use serde::Serialize;
// use time::OffsetDateTime;
// use time::format_description::well_known::Rfc3339;
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::fmt::Write;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 = 100;
const MAX_LARGE_SIZE: u32 = 10_485_760;

thread_local! {
static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static MAP: RefCell<StableBTreeMap<(CallerId, AgentId), Content, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize, CandidType, Default,
)]
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

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize, CandidType, Default,
)]
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
    const MAX_SIZE: u32 = MAX_LARGE_SIZE;

    const IS_FIXED_SIZE: bool = false;
}
// #[derive(Clone)]
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

            // Get existing content or create new
            let mut content = map
                .get(&(caller_id.clone(), agent_id.clone()))
                .unwrap_or_else(|| Content(Vec::new()));

            content.0.push(history_entry);
            map.insert((caller_id, agent_id), content);
        });
    }

    pub fn read_history(caller_id: &String, agent_id: String) -> Vec<(History, History)> {
        let caller_id = CallerId(caller_id.clone());
        let agent_id = AgentId(agent_id);

        MAP.with(|map| {
            let map = map.borrow();
            map.get(&(caller_id, agent_id))
                .map(|content| content.0.clone())
                .unwrap_or_default()
        })
    }

    pub fn clear_history(caller_id: &String, agent_id: String) {
        let caller_id = CallerId(caller_id.clone());
        let agent_id = AgentId(agent_id);

        MAP.with(|map| {
            let mut map = map.borrow_mut();
            map.remove(&(caller_id, agent_id));
        });
    }

    pub fn print_map() -> String {
        let mut history = String::new();
        MAP.with(|map| {
            let map = map.borrow();
            writeln!(history, "Complete Map Contents:").unwrap();

            for ((caller_id, agent_id), content) in map.iter() {
                writeln!(
                    history,
                    "CallerId: {:?}, AgentId: {:?}",
                    caller_id.0, agent_id.0
                )
                .unwrap();
                writeln!(history, "    History Entries:").unwrap();

                for (entry_1, entry_2) in &content.0 {
                    writeln!(
                        history,
                        "      Entry 1 - Role: {:?}, Content: {}",
                        entry_1.role, entry_1.content
                    )
                    .unwrap();
                    // Similar for entry_2
                    writeln!(
                        history,
                        "      Entry 1 - Role: {:?}, Content: {}",
                        entry_2.role, entry_2.content
                    )
                    .unwrap();
                }
            }
        });
        history
    }
}
