
use serde::{Serialize, Deserialize};

use crate::{Index, Storage};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    index: Index,
    storage: Storage,
}

impl AppState {
    pub fn new(index: Index, storage: Storage) -> AppState {
        AppState { index, storage }
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn index_mut(&mut self) -> &mut Index {
        &mut self.index
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut Storage {
        &mut self.storage
    }
}
