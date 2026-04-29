use crate::database_manager::manager::DatabaseManager;
use chrono::{Datelike, Timelike};
use shared::{
    boat_info::BoatInfo,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::{FMS_UPDATE_SECS_INTERVAL, HARBOURMASTER_MMSI},
        types::{FmsError, FmsResult, SatComMessageType, VoyageStatus},
    },
    satcom_message::SatComMessage,
    voyage_order::{VoyageOrder, VoyageOrderBody, VoyageOrderHeader},
};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{
        Notify,
        mpsc::{Receiver, Sender},
    },
    task::JoinHandle,
};

pub struct Fms {
    boats_registry: Arc<BoatsInfoRegistry>,
    database_manager: Arc<std::sync::Mutex<DatabaseManager>>,
    rx: tokio::sync::Mutex<Receiver<SatComMessage>>,
    satcom_tx: Sender<SatComMessage>,
    clock_pulse: Arc<Notify>,
}

impl Fms {
    pub fn init(
        boats_reg: Arc<BoatsInfoRegistry>,
        db_manager: Arc<std::sync::Mutex<DatabaseManager>>,
        rx: Receiver<SatComMessage>,
        satcom_tx: Sender<SatComMessage>,
    ) -> Self {
        Self {
            boats_registry: boats_reg,
            database_manager: db_manager,
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

    async fn respond(
        &self,
        satcom_msg: &SatComMessage,
        msg_type: SatComMessageType,
    ) -> FmsResult<()> {
        let msg: SatComMessage = SatComMessage::new(
            HARBOURMASTER_MMSI,
            *satcom_msg.source(),
            satcom_msg.order_header(),
            msg_type,
            None,
        );

        self.satcom_tx.send(msg).await?;

        Ok(())
    }

    fn update_associated_order_status(
        &self,
        satcom_msg: &SatComMessage,
        new_status: VoyageStatus,
    ) -> FmsResult<()> {
        self.database_manager
            .lock()
            .map_err(|_| FmsError::DatabaseManagerPoisoned)?
            .update_voyage_order_status(*satcom_msg.order_header().id() as i32, new_status)?;

        Ok(())
    }

    async fn update_associated_order_status_and_respond(
        &self,
        satcom_msg: &SatComMessage,
        new_status: VoyageStatus,
        res_msg_type: SatComMessageType,
    ) -> FmsResult<()> {
        self.update_associated_order_status(satcom_msg, new_status)?;
        self.respond(satcom_msg, res_msg_type).await?;

        Ok(())
    }

    async fn run_order_dispatcher(&self) -> FmsResult<()> {
        loop {
            self.clock_pulse.notified().await;

            let unassigned_orders: Box<[VoyageOrder]> = self
                .database_manager
                .lock()
                .map_err(|_| FmsError::DatabaseManagerPoisoned)?
                .get_voyage_orders(None, Some(VoyageStatus::Unassigned), None)?
                .iter()
                .map(|(order, ver, dest)| {
                    let order_header = VoyageOrderHeader::from_data(
                        order.id as u16,
                        order.current_version_number as u8,
                    );

                    let order_body = VoyageOrderBody::from_data(
                        dest.name.to_string(),
                        (dest.longitude as u16, dest.latitude as u16),
                        ver.eta.month() as u8,
                        ver.eta.day() as u8,
                        ver.eta.hour() as u8,
                        ver.eta.minute() as u8,
                        ver.cargo_type as u8,
                        ver.speed_profile as u8,
                    );

                    VoyageOrder::from_components(order_header, order_body)
                })
                .collect();

            let free_boats_mmsis: Box<[u32]> = self
                .boats_registry
                .export()
                .iter()
                .map(|(_, boat)| {
                    if boat.get_voyage_data()?.destination() == "@@@@@@@@@@@@@@@@@@@@" {
                        Ok::<Option<u32>, FmsError>(Some(*boat.get_static_data()?.mmsi()))
                    } else {
                        Ok::<Option<u32>, FmsError>(None)
                    }
                })
                .filter_map(|res: Result<Option<u32>, _>| res.transpose()) // Transforme Result<Option<T>> en Option<Result<T>>
                .collect::<Result<Vec<_>, _>>()? // Collecte et propage l'erreur si besoin
                .into_boxed_slice();

            for i in 0..unassigned_orders.len().min(free_boats_mmsis.len()) {
                let req: SatComMessage = SatComMessage::new(
                    HARBOURMASTER_MMSI,
                    free_boats_mmsis[i],
                    unassigned_orders[i].header(),
                    SatComMessageType::Offer,
                    Some(unassigned_orders[i].body()),
                );

                self.satcom_tx.send(req).await?;

                println!(
                    "Offre pour l'ordre de voyage {} envoyée au bateau {}.",
                    unassigned_orders[i].header().id(),
                    free_boats_mmsis[i]
                );
            }
        }
    }

    async fn handle_offer_ack(
        &self,
        satcom_msg: &SatComMessage,
        concerned_voyage_order: &(
            crate::database_manager::models::VoyageOrderQueryResult,
            crate::database_manager::models::VoyageOrderVersionQueryResult,
            crate::database_manager::models::DestinationQueryResult,
        ),
    ) -> FmsResult<()> {
        self.update_associated_order_status(&satcom_msg, VoyageStatus::UnderRevision)?;

        let concerned_boat_info: BoatInfo = self.boats_registry.get(*satcom_msg.source())?;

        concerned_boat_info.update_voyage_data(
            Some(concerned_voyage_order.2.name.clone()),
            Some(concerned_voyage_order.1.eta.month() as u8),
            Some(concerned_voyage_order.1.eta.day() as u8),
            Some(concerned_voyage_order.1.eta.hour() as u8),
            Some(concerned_voyage_order.1.eta.minute() as u8),
        );

        self.boats_registry.update(concerned_boat_info)?;

        println!(
            "Offre pour l'ordre de voyage {} reçue par le bateau {}. Attente d'une réponse pour la révision initiale.",
            *satcom_msg.order_header().id(),
            *satcom_msg.source()
        );

        Ok(())
    }

    async fn handle_rev_req_ack(&self, satcom_msg: &SatComMessage) -> FmsResult<()> {
        self.update_associated_order_status(&satcom_msg, VoyageStatus::UnderRevision)?;

        println!(
            "Révision de l'ordre de voyage {} demandée par le bateau {}. Attente d'une réponse pour la révision.",
            *satcom_msg.order_header().id(),
            *satcom_msg.source()
        );

        Ok(())
    }

    async fn handle_rev_acceptation(&self, satcom_msg: &SatComMessage) -> FmsResult<()> {
        self.database_manager
            .lock()
            .map_err(|_| FmsError::DatabaseManagerPoisoned)?
            .update_voyage_order_version(
                *satcom_msg.order_header().id() as i32,
                *satcom_msg.order_header().version() as i32,
            )?;

        self.update_associated_order_status_and_respond(
            &satcom_msg,
            VoyageStatus::RevisionAccepted,
            SatComMessageType::Acknowledgement,
        )
        .await?;

        println!(
            "Révision de l'ordre {} acceptée par le bateau {}. Nouvelle version : {}. Accusé de réception envoyé.",
            *satcom_msg.order_header().id(),
            *satcom_msg.source(),
            *satcom_msg.order_header().version()
        );

        Ok(())
    }

    async fn handle_initial_rev_acceptation(&self, satcom_msg: &SatComMessage) -> FmsResult<()> {
        self.database_manager
            .lock()
            .map_err(|_| FmsError::DatabaseManagerPoisoned)?
            .assign_voyage_order(
                *satcom_msg.order_header().id() as i32,
                *satcom_msg.source() as i32,
            )?;

        self.update_associated_order_status_and_respond(
            &satcom_msg,
            VoyageStatus::RevisionAccepted,
            SatComMessageType::Acknowledgement,
        )
        .await?;

        println!(
            "Révision initiale de l'ordre {} acceptée par le bateau {}. Accusé de réception envoyé. Assignation officielle au bateau.",
            *satcom_msg.order_header().id(),
            *satcom_msg.source()
        );

        Ok(())
    }

    async fn handle_rev_refusal(&self, satcom_message: &SatComMessage) -> FmsResult<()> {
        self.database_manager
            .lock()
            .map_err(|_| FmsError::DatabaseManagerPoisoned)?
            .delete_voyage_order_version(
                *satcom_message.order_header().id() as i32,
                *satcom_message.order_header().version() as i32,
            )?;

        self.update_associated_order_status_and_respond(
            &satcom_message,
            VoyageStatus::RevisionRefused,
            SatComMessageType::Acknowledgement,
        )
        .await?;

        println!(
            "Révision de l'ordre {} refusé par le bateau {}. Accusé de réception envoyé.",
            *satcom_message.order_header().id(),
            *satcom_message.source()
        );

        Ok(())
    }

    async fn handle_initial_rev_refusal(&self, satcom_msg: &SatComMessage) -> FmsResult<()> {
        self.update_associated_order_status_and_respond(
            &satcom_msg,
            VoyageStatus::Unassigned,
            SatComMessageType::Acknowledgement,
        )
        .await?;

        println!(
            "Révision initiale de l'ordre {} refusée par le bateau {}. Accusé de réception envoyé. Ordre à nouveau non-assigné.",
            *satcom_msg.order_header().id(),
            *satcom_msg.source()
        );

        Ok(())
    }

    async fn handle_rev_request(&self, satcom_msg: &SatComMessage) -> FmsResult<()> {
        todo!()
    }

    async fn handle_last_agreed_rev_execution(&self, satcom_msg: &SatComMessage) -> FmsResult<()> {
        self.update_associated_order_status_and_respond(
            &satcom_msg,
            VoyageStatus::InExecution,
            SatComMessageType::Acknowledgement,
        )
        .await?;

        println!(
            "Ordre {} en cours d'exécution par le bateau {}. Accusé de réception envoyé.",
            *satcom_msg.order_header().id(),
            *satcom_msg.source()
        );

        Ok(())
    }

    async fn handle_notice_of_readiness(&self, satcom_msg: &SatComMessage) -> FmsResult<()> {
        self.update_associated_order_status_and_respond(
            &satcom_msg,
            VoyageStatus::Completed,
            SatComMessageType::EndOfVoyage,
        )
        .await?;

        println!(
            "Ordre {} achevé par le bateau {}. Notification de fin de voyage envoyée.",
            *satcom_msg.order_header().id(),
            *satcom_msg.source()
        );

        Ok(())
    }

    async fn handle_aborting(&self, satcom_msg: &SatComMessage) -> FmsResult<()> {
        self.update_associated_order_status_and_respond(
            &satcom_msg,
            VoyageStatus::Finished,
            SatComMessageType::Acknowledgement,
        )
        .await?;

        println!(
            "Exécution de l'ordre {} définitivement abandonnée par le bateau {}. Accusé de réception envoyé. Terminaison de l'ordre.",
            *satcom_msg.order_header().id(),
            *satcom_msg.source()
        );

        Ok(())
    }

    async fn run_message_listener(&self) -> FmsResult<()> {
        while let Some(satcom_msg) = self.rx.lock().await.recv().await {
            if *satcom_msg.target() != HARBOURMASTER_MMSI {
                continue;
            }

            let db_order_query_result: Box<
                [(
                    crate::database_manager::models::VoyageOrderQueryResult,
                    crate::database_manager::models::VoyageOrderVersionQueryResult,
                    crate::database_manager::models::DestinationQueryResult,
                )],
            > = self
                .database_manager
                .lock()
                .map_err(|_| FmsError::DatabaseManagerPoisoned)?
                .get_voyage_orders(Some(*satcom_msg.order_header().id() as i32), None, None)
                .unwrap_or(Box::new([]));

            if db_order_query_result.len() != 1 {
                continue;
            }

            let db_order: &(
                crate::database_manager::models::VoyageOrderQueryResult,
                crate::database_manager::models::VoyageOrderVersionQueryResult,
                crate::database_manager::models::DestinationQueryResult,
            ) = &db_order_query_result[0];

            let db_order_status: VoyageStatus = (db_order.0.status as u8).into();
            let db_order_ver_nbr: u8 = db_order.0.current_version_number as u8;

            let db_order_rev: Option<
                crate::database_manager::models::VoyageOrderVersionQueryResult,
            > = self
                .database_manager
                .lock()
                .map_err(|_| FmsError::DatabaseManagerPoisoned)?
                .get_voyage_order_rev_ver(*satcom_msg.order_header().id() as i32)?;

            let mut db_order_rev_ver_nbr: u8 = db_order_ver_nbr + 1;

            if let Some(v) = db_order_rev {
                db_order_rev_ver_nbr = v.version_number as u8;
            }

            let msg_ver: u8 = *satcom_msg.order_header().version();

            let refers_current_ver: bool = msg_ver == db_order_ver_nbr
                && Some(*satcom_msg.source() as i32) == db_order.0.executant;
            let refers_rev_ver: bool = msg_ver == db_order_rev_ver_nbr
                && Some(*satcom_msg.source() as i32) == db_order.0.executant;

            match satcom_msg.message_type() {
                SatComMessageType::Acknowledgement => {
                    if db_order_status == VoyageStatus::Unassigned {
                        self.handle_offer_ack(&satcom_msg, db_order).await?;
                    } else if db_order_status == VoyageStatus::RevisionSubmitted && refers_rev_ver {
                        self.handle_rev_req_ack(&satcom_msg).await?;
                    }
                }
                SatComMessageType::RevisionAcceptation => {
                    if db_order_status == VoyageStatus::UnderRevision && refers_current_ver {
                        self.handle_rev_acceptation(&satcom_msg).await?;
                    } else if db_order_status == VoyageStatus::UnderRevision
                        && db_order_ver_nbr == msg_ver
                        && db_order.0.executant.is_none()
                    {
                        self.handle_initial_rev_acceptation(&satcom_msg).await?;
                    }
                }
                SatComMessageType::RevisionRefusal => {
                    if db_order_status == VoyageStatus::UnderRevision && refers_rev_ver {
                        self.handle_rev_refusal(&satcom_msg).await?;
                    } else if db_order_status == VoyageStatus::UnderRevision
                        && db_order.0.executant.is_none()
                        && msg_ver == db_order_ver_nbr
                    {
                        self.handle_initial_rev_refusal(&satcom_msg).await?;
                    }
                }
                SatComMessageType::RevisionRequest => {
                    if matches!(
                        db_order_status,
                        VoyageStatus::InExecution
                            | VoyageStatus::RevisionAccepted
                            | VoyageStatus::RevisionRefused
                    ) && refers_current_ver
                    {
                        self.handle_rev_request(&satcom_msg).await?;
                    }
                }
                SatComMessageType::ExecutingLastAgreedRevision => {
                    if matches!(
                        db_order_status,
                        VoyageStatus::RevisionAccepted | VoyageStatus::RevisionRefused
                    ) && refers_current_ver
                    {
                        self.handle_last_agreed_rev_execution(&satcom_msg).await?;
                    }
                }
                SatComMessageType::NoticeOfReadiness => {
                    if db_order_status == VoyageStatus::InExecution && refers_current_ver {
                        self.handle_notice_of_readiness(&satcom_msg).await?;
                    }
                }
                SatComMessageType::Aborting => {
                    if refers_current_ver {
                        self.handle_aborting(&satcom_msg).await?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn start(self) -> (JoinHandle<()>, JoinHandle<()>, JoinHandle<()>) {
        let runner_arc: Arc<Self> = Arc::new(self);
        let order_dispatcher_runer_arc: Arc<Self> = runner_arc.clone();
        let clock_runner_arc: Arc<Self> = runner_arc.clone();

        (
            tokio::spawn(async move {
                clock_runner_arc.run_fms_master_clock().await;
            }),
            tokio::spawn(async move {
                match order_dispatcher_runer_arc.run_order_dispatcher().await {
                    Ok(()) => eprintln!("Order dispatcher exited unexpectedly."),
                    Err(e) => eprintln!("Error in order dispatcher: {:?}", e),
                }
            }),
            tokio::spawn(async move {
                match runner_arc.run_message_listener().await {
                    Ok(()) => eprintln!("FMS exited unexpectedly."),
                    Err(e) => eprintln!("Error in message listener: {:?}", e),
                }
            }),
        )
    }
}
