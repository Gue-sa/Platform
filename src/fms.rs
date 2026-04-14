use std::sync::Arc;

use shared::{boats_registry::BoatsInfoRegistry, satcom_message::SatComMessage};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct Fms {
    pub boats_registry: Arc<BoatsInfoRegistry>,
    pub rx: Receiver<SatComMessage>,
    pub satcom_tx: Sender<SatComMessage>,
}

impl Fms {
    pub fn new(
        boats_registry: Arc<BoatsInfoRegistry>,
        rx: Receiver<SatComMessage>,
        satcom_tx: Sender<SatComMessage>,
    ) -> Self {
        Self {
            boats_registry: boats_registry,
            rx: rx,
            satcom_tx: satcom_tx,
        }
    }

    pub async fn start(mut self) -> () {
        todo!()
    }
}
