use std::sync::{Arc, Mutex};

use shared::{boats_registry::BoatsInfoRegistry, satcom_message::SatComMessage};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::database_manager::manager::DatabaseManager;

pub struct Fms {
    pub boats_registry: Arc<BoatsInfoRegistry>,
    pub database_manager: Arc<Mutex<DatabaseManager>>,
    pub rx: Receiver<SatComMessage>,
    pub satcom_tx: Sender<SatComMessage>,
}

impl Fms {
    pub fn new(
        boats_registry: Arc<BoatsInfoRegistry>,
        database_manager: Arc<Mutex<DatabaseManager>>,
        rx: Receiver<SatComMessage>,
        satcom_tx: Sender<SatComMessage>,
    ) -> Self {
        Self {
            boats_registry: boats_registry,
            database_manager: database_manager,
            rx: rx,
            satcom_tx: satcom_tx,
        }
    }

    pub async fn start(mut self) -> () {
        todo!()
    }
}
