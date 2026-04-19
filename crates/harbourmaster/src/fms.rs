use std::{sync::Arc, time::Duration};

use shared::{
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::FMS_UPDATE_SECS_INTERVAL,
        types::{SatComMessageType, VoyageStatus},
    },
    satcom_message::SatComMessage,
};
use tokio::sync::{
    Notify,
    mpsc::{Receiver, Sender},
};

use crate::database_manager::manager::DatabaseManager;

pub struct Fms {
    pub boats_registry: Arc<BoatsInfoRegistry>,
    pub database_manager: Arc<std::sync::Mutex<DatabaseManager>>,
    pub rx: tokio::sync::Mutex<Receiver<SatComMessage>>,
    pub satcom_tx: Sender<SatComMessage>,
    pub clock_pulse: Arc<Notify>,
}

impl Fms {
    pub fn new(
        boats_registry: Arc<BoatsInfoRegistry>,
        database_manager: Arc<std::sync::Mutex<DatabaseManager>>,
        rx: Receiver<SatComMessage>,
        satcom_tx: Sender<SatComMessage>,
    ) -> Self {
        Self {
            boats_registry: boats_registry,
            database_manager: database_manager,
            rx: tokio::sync::Mutex::new(rx),
            satcom_tx: satcom_tx,
            clock_pulse: Arc::new(Notify::new()),
        }
    }

    pub async fn fms_master_clock(&self) -> () {
        loop {
            tokio::time::sleep(Duration::from_secs(FMS_UPDATE_SECS_INTERVAL)).await;
            self.clock_pulse.notify_waiters();
        }
    }

    pub async fn start(mut self) -> () {
        let runner_arc: Arc<Self> = Arc::new(self);
        let clock_runner_arc: Arc<Self> = runner_arc.clone();

        tokio::spawn(async move {
            clock_runner_arc.fms_master_clock().await;

            loop {
                let _ = clock_runner_arc.clock_pulse.notified().await;

                todo!()
            }
        });

        tokio::spawn(async move {
            loop {
                match runner_arc.rx.lock().await.recv().await {
                    Some(satcom_message) => {
                        let concerned_voyage_order_query_result: Box<
                            [(
                                crate::database_manager::models::VoyageOrderQueryResult,
                                crate::database_manager::models::VoyageOrderVersionQueryResult,
                                crate::database_manager::models::DestinationQueryResult,
                            )],
                        > = runner_arc
                            .database_manager
                            .lock()
                            .unwrap()
                            .get_voyage_orders(
                                Some(satcom_message.order_header.id as i32),
                                None,
                                None,
                            )
                            .unwrap_or(Box::new([]));

                        if concerned_voyage_order_query_result.len() == 1 {
                            let concerned_message_order: &(
                                crate::database_manager::models::VoyageOrderQueryResult,
                                crate::database_manager::models::VoyageOrderVersionQueryResult,
                                crate::database_manager::models::DestinationQueryResult,
                            ) = &concerned_voyage_order_query_result[0];

                            let concerned_message_order_status: VoyageStatus =
                                (concerned_message_order.0.status as u8).into();

                            match satcom_message.message_type {
                                SatComMessageType::Acknowledgement => {
                                    if concerned_message_order_status == VoyageStatus::Unassigned {
                                        todo!()
                                    } else if concerned_message_order_status
                                        == VoyageStatus::RevisionSubmitted
                                    {
                                        todo!()
                                    }
                                }
                                SatComMessageType::RevisionAcceptation => {
                                    if concerned_message_order_status
                                        == VoyageStatus::RevisionSubmitted
                                    {
                                        todo!()
                                    }
                                }
                                SatComMessageType::RevisionRefusal => {
                                    if concerned_message_order_status
                                        == VoyageStatus::RevisionSubmitted
                                    {
                                        todo!()
                                    }
                                }
                                SatComMessageType::RevisionRequest => {
                                    if concerned_message_order_status == VoyageStatus::InExecution
                                        || concerned_message_order_status
                                            == VoyageStatus::RevisionAccepted
                                        || concerned_message_order_status
                                            == VoyageStatus::RevisionRefused
                                    {
                                        todo!()
                                    }
                                }
                                SatComMessageType::ExecutingLastAgreedRevision => {
                                    if concerned_message_order_status
                                        == VoyageStatus::RevisionAccepted
                                    {
                                        todo!()
                                    } else if concerned_message_order_status
                                        == VoyageStatus::RevisionRefused
                                    {
                                        todo!()
                                    }
                                }
                                SatComMessageType::NoticeOfReadiness => {
                                    if concerned_message_order_status == VoyageStatus::InExecution {
                                        todo!()
                                    }
                                }
                                SatComMessageType::Aborting => {
                                    todo!()
                                }
                                _ => {}
                            }
                        }
                    }
                    None => {}
                }
            }
        });
    }
}
