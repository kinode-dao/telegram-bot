use serde::{Deserialize, Serialize};
use kinode_process_lib::{get_state, set_state};


#[derive(Debug, Serialize, Deserialize, Default)]
pub struct State {
    pub tg_key: String,
    pub api_url: String,
    pub current_offset: u32,
}

impl State {
    pub fn fetch() -> Option<State> {
        if let Some(state_bytes) = get_state() {
            bincode::deserialize(&state_bytes).ok()
        } else {
            None
        }
    }

    pub fn save(&self) {
        let serialized_state = bincode::serialize(self).expect("Failed to serialize state");
        set_state(&serialized_state);
    }
}
