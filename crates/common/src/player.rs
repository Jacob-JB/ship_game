use serde::{Deserialize, Serialize};

use crate::ServerEntity;

#[derive(Serialize, Deserialize)]
pub struct NewPlayer {
    pub server_entity: ServerEntity,
}
