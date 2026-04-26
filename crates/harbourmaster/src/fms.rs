use std::{sync::Arc, time::Duration};

use chrono::{Datelike, Timelike};
use shared::{
    boat_info::BoatInfo,
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
    boats_registry: Arc<BoatsInfoRegistry>,
    database_manager: Arc<std::sync::Mutex<DatabaseManager>>,
    rx: tokio::sync::Mutex<Receiver<SatComMessage>>,
    satcom_tx: Sender<SatComMessage>,
    clock_pulse: Arc<Notify>,
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

    async fn run_fms_master_clock(&self) -> () {
        loop {
            tokio::time::sleep(Duration::from_secs(FMS_UPDATE_SECS_INTERVAL)).await;
            self.clock_pulse.notify_waiters();
        }
    }

    async fn respond(&self, satcom_message: &SatComMessage, msg_type: SatComMessageType) -> () {
        let msg: SatComMessage = SatComMessage::new(
            HARBOURMASTER_MMSI,
            *satcom_message.source(),
            satcom_message.order_header(),
            msg_type,
            None,
        );

        self.satcom_tx.send(msg).await;
    }

    fn update_associated_order_status(
        &self,
        satcom_message: &SatComMessage,
        new_status: VoyageStatus,
    ) -> () {
        self.database_manager
            .lock()
            .unwrap()
            .update_voyage_order_status(*satcom_message.order_header().id() as i32, new_status);
    }

    async fn update_associated_order_status_and_respond(
        &self,
        satcom_message: &SatComMessage,
        new_status: VoyageStatus,
        response_msg_type: SatComMessageType,
    ) -> () {
        self.update_associated_order_status(satcom_message, new_status);
        self.respond(satcom_message, response_msg_type).await;
    }

    async fn run_order_dispatcher(&self) -> () {
        loop {
            self.clock_pulse.notified().await;

            let unassigned_orders: Box<[VoyageOrder]> = self
                .database_manager
                .lock()
                .unwrap()
                .get_voyage_orders(None, Some(VoyageStatus::Unassigned), None)
                .unwrap()
                .iter()
                .map(|(o, v, d)| {
                    let order_header =
                        VoyageOrderHeader::from(o.id as u16, o.current_version_number as u8);

                    let order_body = VoyageOrderBody::from(
                        d.name.to_string(),
                        (d.longitude as u16, d.latitude as u16),
                        v.eta.month() as u8,
                        v.eta.day() as u8,
                        v.eta.hour() as u8,
                        v.eta.minute() as u8,
                        v.cargo_type as u8,
                        v.speed_profile as u8,
                    );

                    VoyageOrder::from(order_header, order_body)
                })
                .collect();

            let free_boats_mmsis: Box<[u32]> = self
                .boats_registry
                .export()
                .iter()
                .filter(|(_, boat)| boat.get_voyage_data().destination() == "@@@@@@@@@@@@@@@@@@@@")
                .map(|(_, boat)| *boat.get_static_data().mmsi())
                .collect();

            for i in 0..unassigned_orders.len().min(free_boats_mmsis.len()) {
                let req: SatComMessage = SatComMessage::new(
                    HARBOURMASTER_MMSI,
                    free_boats_mmsis[i],
                    unassigned_orders[i].header(),
                    SatComMessageType::Offer,
                    Some(unassigned_orders[i].body()),
                );

                self.satcom_tx.send(req).await;

                println!(
                    "Offre pour l'ordre de voyage {} envoyée au bateau {}.",
                    unassigned_orders[i].header().id(),
                    free_boats_mmsis[i]
                );
            }
        }
    }

    async fn handle_offer_acknowledgement(
        &self,
        satcom_message: &SatComMessage,
        concerned_voyage_order: &(
            crate::database_manager::models::VoyageOrderQueryResult,
            crate::database_manager::models::VoyageOrderVersionQueryResult,
            crate::database_manager::models::DestinationQueryResult,
        ),
    ) -> () {
        self.update_associated_order_status(&satcom_message, VoyageStatus::UnderRevision);

        let concerned_boat_info: BoatInfo =
            self.boats_registry.get(*satcom_message.source()).unwrap();

        concerned_boat_info.update_voyage_data(
            Some(concerned_voyage_order.2.name.clone()),
            Some(concerned_voyage_order.1.eta.month() as u8),
            Some(concerned_voyage_order.1.eta.day() as u8),
            Some(concerned_voyage_order.1.eta.hour() as u8),
            Some(concerned_voyage_order.1.eta.minute() as u8),
        );

        self.boats_registry.update(concerned_boat_info);

        println!(
            "Offre pour l'ordre de voyage {} reçue par le bateau {}. Attente d'une réponse pour la révision initiale.",
            *satcom_message.order_header().id(),
            *satcom_message.source()
        );
    }

    async fn handle_revision_request_acknowledgement(&self, satcom_message: &SatComMessage) -> () {
        self.update_associated_order_status(&satcom_message, VoyageStatus::UnderRevision);

        println!(
            "Révision de l'ordre de voyage {} demandée par le bateau {}. Attente d'une réponse pour la révision.",
            *satcom_message.order_header().id(),
            *satcom_message.source()
        );
    }

    async fn handle_revision_acceptation(&self, satcom_message: &SatComMessage) -> () {
        self.database_manager
            .lock()
            .unwrap()
            .update_voyage_order_version(
                *satcom_message.order_header().id() as i32,
                *satcom_message.order_header().version() as i32,
            );

        self.update_associated_order_status_and_respond(
            &satcom_message,
            VoyageStatus::RevisionAccepted,
            SatComMessageType::Acknowledgement,
        )
        .await;

        println!(
            "Révision de l'ordre {} acceptée par le bateau {}. Nouvelle version : {}. Accusé de réception envoyé.",
            *satcom_message.order_header().id(),
            *satcom_message.source(),
            *satcom_message.order_header().version()
        );
    }

    async fn handle_initial_revision_acceptation(&self, satcom_message: &SatComMessage) -> () {
        self.database_manager.lock().unwrap().assign_voyage_order(
            *satcom_message.order_header().id() as i32,
            *satcom_message.source() as i32,
        );

        self.update_associated_order_status_and_respond(
            &satcom_message,
            VoyageStatus::RevisionAccepted,
            SatComMessageType::Acknowledgement,
        )
        .await;

        println!(
            "Révision initiale de l'ordre {} acceptée par le bateau {}. Accusé de réception envoyé. Assignation officielle au bateau.",
            *satcom_message.order_header().id(),
            *satcom_message.source()
        );
    }

    async fn handle_revision_refusal(&self, satcom_message: &SatComMessage) -> () {
        self.database_manager
            .lock()
            .unwrap()
            .delete_voyage_order_version(
                *satcom_message.order_header().id() as i32,
                *satcom_message.order_header().version() as i32,
            );

        self.update_associated_order_status_and_respond(
            &satcom_message,
            VoyageStatus::RevisionRefused,
            SatComMessageType::Acknowledgement,
        )
        .await;

        println!(
            "Révision de l'ordre {} refusé par le bateau {}. Accusé de réception envoyé.",
            *satcom_message.order_header().id(),
            *satcom_message.source()
        );
    }

    async fn handle_initial_revision_refusal(&self, satcom_message: &SatComMessage) -> () {
        self.update_associated_order_status_and_respond(
            &satcom_message,
            VoyageStatus::Unassigned,
            SatComMessageType::Acknowledgement,
        )
        .await;

        println!(
            "Révision initiale de l'ordre {} refusée par le bateau {}. Accusé de réception envoyé. Ordre à nouveau non-assigné.",
            *satcom_message.order_header().id(),
            *satcom_message.source()
        );
    }

    async fn handle_revision_request(&self, satcom_message: &SatComMessage) -> () {
        todo!()
    }

    async fn handle_last_agreed_revision_execution(&self, satcom_message: &SatComMessage) -> () {
        self.update_associated_order_status_and_respond(
            &satcom_message,
            VoyageStatus::InExecution,
            SatComMessageType::Acknowledgement,
        )
        .await;

        println!(
            "Ordre {} en cours d'exécution par le bateau {}. Accusé de réception envoyé.",
            *satcom_message.order_header().id(),
            *satcom_message.source()
        );
    }

    async fn handle_notice_of_readiness(&self, satcom_message: &SatComMessage) -> () {
        self.update_associated_order_status_and_respond(
            &satcom_message,
            VoyageStatus::Completed,
            SatComMessageType::EndOfVoyage,
        )
        .await;

        println!(
            "Ordre {} achevé par le bateau {}. Notification de fin de voyage envoyée.",
            *satcom_message.order_header().id(),
            *satcom_message.source()
        );
    }

    async fn handle_aborting(&self, satcom_message: &SatComMessage) -> () {
        self.update_associated_order_status_and_respond(
            &satcom_message,
            VoyageStatus::Finished,
            SatComMessageType::Acknowledgement,
        )
        .await;

        println!(
            "Exécution de l'ordre {} définitivement abandonnée par le bateau {}. Accusé de réception envoyé. Terminaison de l'ordre.",
            *satcom_message.order_header().id(),
            *satcom_message.source()
        );
    }

    async fn run_message_listener(&self) -> () {
        loop {
            let satcom_message = match self.rx.lock().await.recv().await {
                Some(m) => m,
                None => break,
            };

            if *satcom_message.target() != HARBOURMASTER_MMSI {
                continue;
            }

            let concerned_voyage_order_query_result: Box<
                [(
                    crate::database_manager::models::VoyageOrderQueryResult,
                    crate::database_manager::models::VoyageOrderVersionQueryResult,
                    crate::database_manager::models::DestinationQueryResult,
                )],
            > = self
                .database_manager
                .lock()
                .unwrap()
                .get_voyage_orders(Some(*satcom_message.order_header().id() as i32), None, None)
                .unwrap_or(Box::new([]));

            if concerned_voyage_order_query_result.len() != 1 {
                continue;
            }

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
            > = self
                .database_manager
                .lock()
                .unwrap()
                .get_voyage_order_revision_version(*satcom_message.order_header().id() as i32)
                .unwrap();

            let mut concerned_voyage_order_revision_version_number: u8 =
                concerned_voyage_order_version_number + 1;

            if let Some(v) = concerned_voyage_order_revision {
                concerned_voyage_order_revision_version_number = v.version_number as u8;
            }

            let msg_version: u8 = *satcom_message.order_header().version();

            let corresponds_to_current_version: bool = msg_version
                == concerned_voyage_order_version_number
                && Some(*satcom_message.source() as i32) == concerned_voyage_order.0.executant;
            let corresponds_to_revision_version: bool = msg_version
                == concerned_voyage_order_revision_version_number
                && Some(*satcom_message.source() as i32) == concerned_voyage_order.0.executant;

            match satcom_message.message_type() {
                SatComMessageType::Acknowledgement => {
                    if concerned_voyage_order_status == VoyageStatus::Unassigned {
                        self.handle_offer_acknowledgement(&satcom_message, concerned_voyage_order)
                            .await;
                    } else if concerned_voyage_order_status == VoyageStatus::RevisionSubmitted
                        && corresponds_to_revision_version
                    {
                        self.handle_revision_request_acknowledgement(&satcom_message)
                            .await;
                    }
                }
                SatComMessageType::RevisionAcceptation => {
                    if concerned_voyage_order_status == VoyageStatus::UnderRevision
                        && corresponds_to_current_version
                    {
                        self.handle_revision_acceptation(&satcom_message).await;
                    } else if concerned_voyage_order_status == VoyageStatus::UnderRevision
                        && concerned_voyage_order_version_number == msg_version
                        && concerned_voyage_order.0.executant.is_none()
                    {
                        self.handle_initial_revision_acceptation(&satcom_message)
                            .await;
                    }
                }
                SatComMessageType::RevisionRefusal => {
                    if concerned_voyage_order_status == VoyageStatus::UnderRevision
                        && corresponds_to_revision_version
                    {
                        self.handle_revision_refusal(&satcom_message).await;
                    } else if concerned_voyage_order_status == VoyageStatus::UnderRevision
                        && concerned_voyage_order.0.executant.is_none()
                        && msg_version == concerned_voyage_order_version_number
                    {
                        self.handle_initial_revision_refusal(&satcom_message).await;
                    }
                }
                SatComMessageType::RevisionRequest => {
                    if matches!(
                        concerned_voyage_order_status,
                        VoyageStatus::InExecution
                            | VoyageStatus::RevisionAccepted
                            | VoyageStatus::RevisionRefused
                    ) && corresponds_to_current_version
                    {
                        self.handle_revision_request(&satcom_message).await;
                    }
                }
                SatComMessageType::ExecutingLastAgreedRevision => {
                    if matches!(
                        concerned_voyage_order_status,
                        VoyageStatus::RevisionAccepted | VoyageStatus::RevisionRefused
                    ) && corresponds_to_current_version
                    {
                        self.handle_last_agreed_revision_execution(&satcom_message)
                            .await;
                    }
                }
                SatComMessageType::NoticeOfReadiness => {
                    if concerned_voyage_order_status == VoyageStatus::InExecution
                        && corresponds_to_current_version
                    {
                        self.handle_notice_of_readiness(&satcom_message).await;
                    }
                }
                SatComMessageType::Aborting => {
                    if corresponds_to_current_version {
                        self.handle_aborting(&satcom_message).await;
                    }
                }
                _ => {}
            }
        }
    }

    pub async fn start(self) -> () {
        let runner_arc: Arc<Self> = Arc::new(self);
        let order_dispatcher_runer_arc: Arc<Self> = runner_arc.clone();
        let clock_runner_arc: Arc<Self> = runner_arc.clone();

        tokio::spawn(async move {
            clock_runner_arc.run_fms_master_clock().await;
        });

        tokio::spawn(async move {
            order_dispatcher_runer_arc.run_order_dispatcher().await;
        });

        tokio::spawn(async move {
            runner_arc.run_message_listener().await;
        });
    }
}
