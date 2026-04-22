use std::{sync::Arc, time::Duration};

use chrono::{Datelike, Timelike};
use shared::{
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::{FMS_UPDATE_SECS_INTERVAL, HARBOURMASTER_MMSI},
        types::{SatComMessageType, VoyageStatus},
    },
    satcom_message::SatComMessage,
    voyage_order::{VoyageOrder, VoyageOrderBody, VoyageOrderHeader},
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
        let listener_satcom_tx_clone: Sender<SatComMessage> = self.satcom_tx.clone();
        let sender_satcom_tx_clone: Sender<SatComMessage> = self.satcom_tx.clone();
        let database_manager_listener_clone: Arc<std::sync::Mutex<DatabaseManager>> =
            self.database_manager.clone();
        let database_manager_sender_clone: Arc<std::sync::Mutex<DatabaseManager>> =
            self.database_manager.clone();
        let boats_registry_clone: Arc<BoatsInfoRegistry> = self.boats_registry.clone();

        let clock_pulse_clone: Arc<Notify> = self.clock_pulse.clone();
        let runner_arc: Arc<Self> = Arc::new(self);
        let clock_runner_arc: Arc<Self> = runner_arc.clone();

        tokio::spawn(async move {
            clock_runner_arc.fms_master_clock().await;
        });

        tokio::spawn(async move {
            loop {
                let _ = clock_pulse_clone.notified().await;
                let unassigned_orders_query_result: Box<[(crate::database_manager::models::VoyageOrderQueryResult, crate::database_manager::models::VoyageOrderVersionQueryResult, crate::database_manager::models::DestinationQueryResult)]> = database_manager_sender_clone.lock().unwrap().get_voyage_orders(None, Some(VoyageStatus::Unassigned), None).unwrap();
                
                let unassigned_orders: Box<[VoyageOrder]> = unassigned_orders_query_result.iter().map(|(o, v, d)| {
                    VoyageOrder {
                        header: VoyageOrderHeader {
                            id: o.id as u16,
                            version: o.current_version_number as u8
                        }, body: VoyageOrderBody {
                            destination: d.name.to_string(),
                            destination_position: (d.longitude as u16, d.latitude as u16),
                            eta_month: v.eta.month() as u8,
                            eta_day: v.eta.day() as u8,
                            eta_hour: v.eta.hour() as u8,
                            eta_minute: v.eta.minute() as u8,
                            cargo_type: v.cargo_type as u8,
                            speed_profile: v.speed_profile as u8
                        }
                    }
                }).collect();
                
                let free_boats_mmsi: Box<[u32]> = boats_registry_clone
                    .export()
                    .iter()
                    .filter(|(_, boat)| boat.get_voyage_data().destination == "@@@@@@@@@@@@@@@@@@@@")
                    .map(|(_, boat)| boat.get_static_data().mmsi)
                    .collect();

                for i in 0..unassigned_orders.len().min(free_boats_mmsi.len()) {
                    let req: SatComMessage = SatComMessage::new(
                        HARBOURMASTER_MMSI,
                        free_boats_mmsi[i],
                        unassigned_orders[i].clone().header,
                        SatComMessageType::Offer,
                        Some(unassigned_orders[i].clone().body)
                    );

                    sender_satcom_tx_clone.send(req.clone()).await;

                    println!("Offre pour l'ordre de voyage {} envoyée au bateau {}.", req.order_header.id, free_boats_mmsi[i]);
                }
            }
        });

        tokio::spawn(async move {
            loop {
                match runner_arc.rx.lock().await.recv().await {
                    Some(satcom_message) => {
                        if satcom_message.target == HARBOURMASTER_MMSI {
                            println!("{:#?}", satcom_message);
                            let mut msg_template: SatComMessage = SatComMessage::new(
                                HARBOURMASTER_MMSI,
                                satcom_message.source,
                                satcom_message.order_header.clone(),
                                SatComMessageType::Acknowledgement,
                                None,
                            );

                            let satcom_message_order_header: VoyageOrderHeader =
                                satcom_message.clone().order_header;

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
                                    Some(satcom_message_order_header.id as i32),
                                    None,
                                    None,
                                )
                                .unwrap_or(Box::new([]));

                            if concerned_voyage_order_query_result.len() == 1 {
                                let concerned_voyage_order: &(
                                    crate::database_manager::models::VoyageOrderQueryResult,
                                    crate::database_manager::models::VoyageOrderVersionQueryResult,
                                    crate::database_manager::models::DestinationQueryResult,
                                ) = &concerned_voyage_order_query_result[0];

                                let concerned_voyage_order_status: VoyageStatus =
                                    (concerned_voyage_order.0.status as u8).into();
                                let concerned_voyage_order_version_number: u8 =
                                    concerned_voyage_order.0.current_version_number as u8;

                                let concerned_voyage_order_revision: Option<
                                    crate::database_manager::models::VoyageOrderVersionQueryResult,
                                > = database_manager_listener_clone
                                    .lock()
                                    .unwrap()
                                    .get_voyage_order_revision_version(
                                        satcom_message_order_header.id as i32,
                                    )
                                    .unwrap();

                                let mut concerned_voyage_order_revision_version_number: u8 =
                                    concerned_voyage_order_version_number + 1;

                                if let Some(v) = concerned_voyage_order_revision {
                                    concerned_voyage_order_revision_version_number =
                                        v.version_number as u8;
                                }

                                println!("{:?}", concerned_voyage_order.0.executant.is_none());
                                println!("{:?}", concerned_voyage_order_version_number == concerned_voyage_order_version_number);
                                println!("{:?}", concerned_voyage_order_status == VoyageStatus::UnderRevision);

                                match satcom_message.message_type {
                                    SatComMessageType::Acknowledgement => {
                                        if concerned_voyage_order_status == VoyageStatus::Unassigned
                                            || (concerned_voyage_order_status
                                                == VoyageStatus::RevisionSubmitted
                                                && Some(satcom_message.source as i32)
                                                    == concerned_voyage_order.0.executant && satcom_message_order_header.version == concerned_voyage_order_revision_version_number)
                                        {
                                            database_manager_listener_clone
                                                .lock()
                                                .unwrap()
                                                .update_voyage_order_status(
                                                    satcom_message_order_header.id as i32,
                                                    VoyageStatus::UnderRevision,
                                                );

                                                println!("Offre pour l'ordre de voyage {} reçue par le bateau {}. Attente d'une réponse pour la révision initiale.", satcom_message.order_header.id, satcom_message.source);
                                        }
                                    }
                                    SatComMessageType::RevisionAcceptation => {
                                        if concerned_voyage_order_status
                                            == VoyageStatus::UnderRevision
                                            && (Some(satcom_message.source as i32)
                                                == concerned_voyage_order.0.executant && satcom_message_order_header.version == concerned_voyage_order_revision_version_number)
                                        {
                                            database_manager_listener_clone
                                                .lock()
                                                .unwrap()
                                                .update_voyage_order_status(
                                                    satcom_message_order_header.id as i32,
                                                    VoyageStatus::RevisionAccepted,
                                                );

                                            database_manager_listener_clone
                                                .lock()
                                                .unwrap()
                                                .update_voyage_order_version(
                                                    satcom_message_order_header.id as i32,
                                                    satcom_message_order_header.version as i32,
                                                );

                                            listener_satcom_tx_clone.send(msg_template).await;

                                            println!("Révision de l'ordre {} acceptée par le bateau {}. Nouvelle version : {}. Accusé de réception envoyé.", satcom_message.order_header.id, satcom_message.order_header.version, satcom_message.source);
                                        } else if concerned_voyage_order_status
                                            == VoyageStatus::UnderRevision
                                            && (concerned_voyage_order.0.executant.is_none() && satcom_message_order_header.version == concerned_voyage_order_version_number) {
                                                database_manager_listener_clone
                                                    .lock()
                                                    .unwrap()
                                                    .update_voyage_order_status(
                                                        satcom_message_order_header.id as i32,
                                                        VoyageStatus::RevisionAccepted,
                                                    );

                                                database_manager_listener_clone
                                                    .lock()
                                                    .unwrap()
                                                    .assign_voyage_order(
                                                        satcom_message_order_header.id as i32,
                                                        satcom_message.source as i32,
                                                    );

                                                listener_satcom_tx_clone.send(msg_template).await;

                                                println!("Révision initiale de l'ordre {} acceptée par le bateau {}. Accusé de réception envoyé. Assignation officielle au bateau.", satcom_message.order_header.id, satcom_message.source);
                                            }
                                        }
                                    SatComMessageType::RevisionRefusal => {
                                        if concerned_voyage_order_status
                                            == VoyageStatus::UnderRevision
                                            && (Some(satcom_message.source as i32)
                                                == concerned_voyage_order.0.executant && satcom_message_order_header.version == concerned_voyage_order_revision_version_number)
                                        {
                                            database_manager_listener_clone
                                                .lock()
                                                .unwrap()
                                                .update_voyage_order_status(
                                                    satcom_message_order_header.id as i32,
                                                    VoyageStatus::RevisionRefused,
                                                );

                                            database_manager_listener_clone
                                                .lock()
                                                .unwrap()
                                                .delete_voyage_order_version(
                                                    satcom_message_order_header.id as i32,
                                                    satcom_message_order_header.version as i32,
                                                );

                                            listener_satcom_tx_clone.send(msg_template).await;

                                            println!("Révision de l'ordre {} refusé par le bateau {}. Accusé de réception envoyé.", satcom_message.order_header.id, satcom_message.source);
                                        }  else if concerned_voyage_order_status
                                            == VoyageStatus::UnderRevision
                                            && (concerned_voyage_order.0.executant.is_none() && satcom_message_order_header.version == concerned_voyage_order_version_number) {
                                                database_manager_listener_clone
                                                    .lock()
                                                    .unwrap()
                                                    .update_voyage_order_status(
                                                        satcom_message_order_header.id as i32,
                                                        VoyageStatus::Unassigned,
                                                    );

                                                listener_satcom_tx_clone.send(msg_template).await;

                                                println!("Révision initale de l'ordre {} refusée par le bateau {}. Accusé de réception envoyé. Ordre à nouveau non-assigné.", satcom_message.order_header.id, satcom_message.source);
                                            }
                                    }
                                    SatComMessageType::RevisionRequest => {
                                        if (concerned_voyage_order_status
                                            == VoyageStatus::InExecution
                                            || concerned_voyage_order_status
                                                == VoyageStatus::RevisionAccepted
                                            || concerned_voyage_order_status
                                                == VoyageStatus::RevisionRefused)
                                            && Some(satcom_message.source as i32)
                                                == concerned_voyage_order.0.executant && satcom_message_order_header.version == concerned_voyage_order_version_number
                                        {
                                            todo!()
                                        }
                                    }
                                    SatComMessageType::ExecutingLastAgreedRevision => {
                                        if (concerned_voyage_order_status
                                                == VoyageStatus::RevisionAccepted
                                            || concerned_voyage_order_status
                                                == VoyageStatus::RevisionRefused)
                                            && Some(satcom_message.source as i32)
                                                == concerned_voyage_order.0.executant && satcom_message_order_header.version == concerned_voyage_order_version_number
                                        {
                                            database_manager_listener_clone
                                                .lock()
                                                .unwrap()
                                                .update_voyage_order_status(
                                                    satcom_message_order_header.id as i32,
                                                    VoyageStatus::InExecution,
                                                );

                                            listener_satcom_tx_clone.send(msg_template).await;

                                            println!("Ordre {} en cours d'exécution par le bateau {}. Accusé de réception envoyé.", satcom_message.order_header.id, satcom_message.source);
                                        }
                                    }
                                    SatComMessageType::NoticeOfReadiness => {
                                        if concerned_voyage_order_status
                                            == VoyageStatus::InExecution
                                            && Some(satcom_message.source as i32)
                                                == concerned_voyage_order.0.executant && satcom_message_order_header.version == concerned_voyage_order_version_number
                                        {
                                            database_manager_listener_clone
                                                .lock()
                                                .unwrap()
                                                .update_voyage_order_status(
                                                    satcom_message_order_header.id as i32,
                                                    VoyageStatus::Completed,
                                                );
                                            
                                            msg_template.message_type = SatComMessageType::EndOfVoyage;
                                            listener_satcom_tx_clone.send(msg_template).await;

                                            println!("Ordre {} achevé par le bateau {}. Notification de fin de voyage envoyée.", satcom_message.order_header.id, satcom_message.source);
                                        }
                                    }
                                    SatComMessageType::Aborting => {
                                        if Some(satcom_message.source as i32)
                                            == concerned_voyage_order.0.executant && satcom_message_order_header.version == concerned_voyage_order_version_number
                                        {
                                            database_manager_listener_clone
                                                .lock()
                                                .unwrap()
                                                .update_voyage_order_status(
                                                    satcom_message_order_header.id as i32,
                                                    VoyageStatus::Finished,
                                                );

                                            listener_satcom_tx_clone.send(msg_template).await;

                                            println!("Exécution de l'ordre {} définitivement abandonée par le bateau {}. Accusé de réception envoyé. Terminaison de l'ordre.", satcom_message.order_header.id, satcom_message.source);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    None => {}
                }
            }
        });
    }
}
