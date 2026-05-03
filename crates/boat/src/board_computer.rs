use crate::voyage::Voyage;
use colored::*;
use shared::{
    boat_info::BoatInfo,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::HARBOURMASTER_MMSI,
        errors::{BoardComputerError, BoardComputerResult},
        types::{LogEvent, SatComMessageType, VoyageStatus},
    },
    satcom_message::SatComMessage,
    voyage_order::{VoyageOrder, VoyageOrderBody, VoyageOrderHeader},
};
use std::sync::Arc;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub struct BoardComputer {
    boat_info: Arc<BoatInfo>,
    boats_registry: Arc<BoatsInfoRegistry>,
    voyage: Option<Voyage>,
    rx: Receiver<SatComMessage>,
    satcom_tx: Sender<SatComMessage>,
    voyage_order_revision: Option<VoyageOrder>,
    logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
}

impl BoardComputer {
    pub fn init(
        boat_info: Arc<BoatInfo>,
        boats_registry: Arc<BoatsInfoRegistry>,
        voyage: Option<Voyage>,
        rx: Receiver<SatComMessage>,
        satcom_tx: Sender<SatComMessage>,
        logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
    ) -> Self {
        Self {
            boat_info: boat_info,
            boats_registry: boats_registry,
            voyage: voyage,
            rx: rx,
            satcom_tx: satcom_tx,
            voyage_order_revision: None,
            logs_cli_tx: logs_cli_tx,
        }
    }

    fn logs_cli_tx(&self) -> std::sync::mpsc::Sender<LogEvent> {
        self.logs_cli_tx.clone()
    }

    fn order(&self) -> BoardComputerResult<&VoyageOrder> {
        Ok(self
            .voyage
            .as_ref()
            .ok_or(BoardComputerError::NoVoyageOrder)?
            .order())
    }

    fn order_revision(&self) -> BoardComputerResult<&VoyageOrder> {
        Ok(self
            .voyage_order_revision
            .as_ref()
            .ok_or(BoardComputerError::NoVoyageOrderRevision)?)
    }

    fn order_revision_mut(&mut self) -> BoardComputerResult<&mut VoyageOrder> {
        Ok(self
            .voyage_order_revision
            .as_mut()
            .ok_or(BoardComputerError::NoVoyageOrderRevision)?)
    }

    fn has_voyage(&self) -> bool {
        self.voyage.is_some()
    }

    fn update_voyage(&mut self, new_voyage: Voyage) -> () {
        self.voyage = Some(new_voyage);
    }

    fn update_voyage_status(&mut self, status: VoyageStatus) -> BoardComputerResult<()> {
        if let Some(ref mut voyage) = self.voyage {
            voyage.set_status(status);

            Ok(())
        } else {
            Err(BoardComputerError::NoVoyageOrder)
        }
    }

    fn matches_status(&self, status: Option<VoyageStatus>) -> bool {
        match (self.voyage.as_ref(), status) {
            (Some(v), Some(s)) => *v.status() == s,
            (None, None) => true,
            _ => false,
        }
    }

    fn adopt_voyage_order(&mut self, order: &VoyageOrder) -> BoardComputerResult<()> {
        self.update_voyage(Voyage::from(
            order.clone(),
            (
                *self.boat_info.get_navigation_data()?.longitude() as u16, // ATTENTION, CE N'EST PAS CORRECT, MAIS ON PART DU PRINCIPE QUE COMME ON UTILISER UN REPERE 1920x1080, UN u16 SUFFIT !
                *self.boat_info.get_navigation_data()?.latitude() as u16,
            ),
        ));

        let order_body = order.body();

        self.boat_info.update_voyage_data(
            Some(order_body.destination().to_string()),
            Some(*order_body.eta_month()),
            Some(*order_body.eta_day()),
            Some(*order_body.eta_hour()),
            Some(*order_body.eta_minute()),
        )?;

        Ok(())
    }

    fn adopt_voyage_order_rev(&mut self) -> BoardComputerResult<()> {
        if let Some(order) = self.voyage_order_revision.take() {
            self.adopt_voyage_order(&order)?;

            Ok(())
        } else {
            Err(BoardComputerError::NoVoyageOrderRevision)
        }
    }

    async fn respond(
        &mut self,
        satcom_msg: &SatComMessage,
        msg_type: SatComMessageType,
        res_order_header: Option<VoyageOrderHeader>,
        res_order_revision: Option<VoyageOrderBody>,
    ) -> BoardComputerResult<()> {
        let message = SatComMessage::new(
            *self.boat_info.get_static_data()?.mmsi(),
            HARBOURMASTER_MMSI,
            res_order_header.unwrap_or(satcom_msg.order_header()),
            msg_type,
            res_order_revision,
        );

        self.satcom_tx.send(message).await?;

        Ok(())
    }

    async fn update_voyage_status_and_respond(
        &mut self,
        new_status: VoyageStatus,
        satcom_msg: &SatComMessage,
        msg_type: SatComMessageType,
        res_order_header: Option<VoyageOrderHeader>,
        res_order_rev: Option<VoyageOrderBody>,
        log_msg: String,
    ) -> BoardComputerResult<()> {
        self.update_voyage_status(new_status)?;

        self.respond(satcom_msg, msg_type, res_order_header, res_order_rev)
            .await?;

        self.logs_cli_tx().send(LogEvent::Computer(log_msg.cyan()));

        Ok(())
    }

    async fn handle_offer(&mut self, satcom_msg: &SatComMessage) -> BoardComputerResult<()> {
        self.respond(satcom_msg, SatComMessageType::Acknowledgement, None, None)
            .await?;

        self.logs_cli_tx().send(LogEvent::Computer(
            format!(
                "Offre d'ordre de voyage reçue (ID {}). Accusé de réception envoyé.",
                satcom_msg.order_header().id()
            )
            .cyan(),
        ));

        if !self.has_voyage() {
            let voyage_order = &satcom_msg
                .order()
                .ok_or(BoardComputerError::NoVoyageOrder)?;

            self.adopt_voyage_order(voyage_order)?;

            self.update_voyage_status_and_respond(
                VoyageStatus::RevisionAccepted,
                satcom_msg,
                SatComMessageType::RevisionAcceptation,
                None,
                None,
                format!(
                    "Ordre de voyage {} accepté.",
                    satcom_msg.order_header().id()
                ),
            )
            .await?;
        } else {
            self.respond(satcom_msg, SatComMessageType::RevisionRefusal, None, None)
                .await?;

            self.logs_cli_tx().send(LogEvent::Computer(
                format!(
                    "Ordre de voyage {} refusé (navire déjà en activité).",
                    satcom_msg.order_header().id()
                )
                .cyan(),
            ));
        }

        Ok(())
    }

    async fn handle_rev_req_ack(&mut self, satcom_msg: &SatComMessage) -> BoardComputerResult<()> {
        self.order_revision_mut()?
            .set_ver(*satcom_msg.order_header().version());

        self.update_voyage_status(VoyageStatus::UnderRevision)?;

        self.logs_cli_tx().send(LogEvent::Computer(format!(
            "Demande de révision de l'ordre {} reçu par la capitainerie. Attente d'une réponse.",
            satcom_msg.order_header().id()
        )
        .cyan()));

        Ok(())
    }

    async fn handle_initial_rev_acceptation_ack(
        &mut self,
        satcom_msg: &SatComMessage,
    ) -> BoardComputerResult<()> {
        self.update_voyage_status_and_respond(
            VoyageStatus::InExecution,
            satcom_msg,
            SatComMessageType::ExecutingLastAgreedRevision,
            None,
            None,
            format!(
                "Ordre de voyage {} en cours d'exécution.",
                satcom_msg.order_header().id()
            ),
        )
        .await?;

        Ok(())
    }

    async fn handle_rev_acceptation(
        &mut self,
        satcom_msg: &SatComMessage,
    ) -> BoardComputerResult<()> {
        self.order_revision_mut()?
            .set_ver(*satcom_msg.order_header().version());

        self.update_voyage_status_and_respond(
                VoyageStatus::RevisionAccepted,
                satcom_msg,
                SatComMessageType::RevisionAcceptation,
                None,
                None,
                format!(
                    "Révision de l'ordre de voyage {} acceptée. Nouvelle version : n°{}. Exécution en cours.",
                    satcom_msg.order_header().id(), satcom_msg.order_header().version()
                )
            )
            .await?;

        Ok(())
    }

    async fn handle_rev_refusal(&mut self, satcom_msg: &SatComMessage) -> BoardComputerResult<()> {
        self.update_voyage_status_and_respond(
                VoyageStatus::RevisionRefused,
                satcom_msg,
                SatComMessageType::RevisionRefusal,
                None,
                None,
                format!(
                    "Révision de l'ordre de voyage {} refusée. Retour à la version n°{}. Exécution en cours.",
                    satcom_msg.order_header().id(), satcom_msg.order_header().version()
                )
            )
            .await?;

        Ok(())
    }

    async fn handle_rev_request(&mut self, satcom_msg: &SatComMessage) -> BoardComputerResult<()> {
        self.voyage_order_revision = Some(
            satcom_msg
                .order()
                .ok_or(BoardComputerError::NoVoyageOrderRevision)?,
        );

        self.update_voyage_status_and_respond(
            VoyageStatus::UnderRevision,
            satcom_msg,
            SatComMessageType::Acknowledgement,
            None,
            None,
            format!(
                "Demande de révision de l'ordre de voyage {} reçue. Traitement de la demande.",
                satcom_msg.order_header().id()
            ),
        )
        .await?;

        self.adopt_voyage_order_rev()?;

        self.update_voyage_status_and_respond(
                VoyageStatus::RevisionAccepted,
                satcom_msg,
                SatComMessageType::RevisionAcceptation,
                None,
                None,
                format!(
                    "Révision de l'ordre de voyage {} acceptée. Nouvelle version : n°{}. Exécution en cours.",
                    satcom_msg.order_header().id(), satcom_msg.order_header().version()
                )
            )
            .await?;

        Ok(())
    }

    async fn handle_end_of_voyage(
        &mut self,
        satcom_msg: &SatComMessage,
    ) -> BoardComputerResult<()> {
        self.update_voyage_status_and_respond(
            VoyageStatus::Finished,
            satcom_msg,
            SatComMessageType::Acknowledgement,
            None,
            None,
            format!(
                "Ordre de voyage {} achevé. Attente d'un nouvel ordre.",
                satcom_msg.order_header().id()
            ),
        )
        .await?;

        Ok(())
    }

    async fn run_board_computer(&mut self) -> BoardComputerResult<()> {
        self.logs_cli_tx().send(LogEvent::System(
            "Lancement de l'ordinateur de bord...".yellow(),
        ));

        let self_mmsi = *self.boat_info.get_static_data()?.mmsi();

        while let Some(satcom_msg) = self.rx.recv().await {
            if *satcom_msg.target() != self_mmsi || *satcom_msg.source() != HARBOURMASTER_MMSI {
                continue;
            }

            let concerns_current_voyage = self
                .voyage
                .as_ref()
                .map_or(false, |v| v.order().header() == satcom_msg.order_header());

            let concerns_current_rev = self
                .voyage_order_revision
                .as_ref()
                .map_or(false, |rev| rev.header() == satcom_msg.order_header());

            match *satcom_msg.message_type() {
                SatComMessageType::Offer => {
                    self.handle_offer(&satcom_msg).await?;
                }
                SatComMessageType::Acknowledgement => {
                    if self.matches_status(Some(VoyageStatus::RevisionSubmitted))
                        && concerns_current_rev
                    {
                        self.handle_rev_req_ack(&satcom_msg).await?;
                    } else if self.matches_status(Some(VoyageStatus::RevisionAccepted))
                        && concerns_current_voyage
                    {
                        self.handle_initial_rev_acceptation_ack(&satcom_msg).await?;
                    }
                }
                SatComMessageType::RevisionAcceptation => {
                    if self.matches_status(Some(VoyageStatus::RevisionSubmitted))
                        && concerns_current_rev
                    {
                        self.handle_rev_acceptation(&satcom_msg).await?;
                    }
                }
                SatComMessageType::RevisionRefusal => {
                    if self.matches_status(Some(VoyageStatus::RevisionSubmitted))
                        && concerns_current_rev
                    {
                        self.handle_rev_refusal(&satcom_msg).await?;
                    }
                }
                SatComMessageType::RevisionRequest => {
                    if (self.matches_status(Some(VoyageStatus::RevisionAccepted))
                        || self.matches_status(Some(VoyageStatus::RevisionRefused))
                        || self.matches_status(Some(VoyageStatus::InExecution)))
                        && *self.order()?.header().id() == *satcom_msg.order_header().id()
                    {
                        self.handle_rev_request(&satcom_msg).await?;
                    }
                }
                SatComMessageType::EndOfVoyage => {
                    if self.matches_status(Some(VoyageStatus::Completed)) && concerns_current_voyage
                    {
                        self.handle_end_of_voyage(&satcom_msg).await?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn start(mut self) -> JoinHandle<()> {
        // ATTENTION : tout ce qui touche à la révision d'ordres de voyage en cours de route est très hasardeux, pour ne pas dire 0% fonctionnel.
        tokio::spawn(async move {
            let _ = self.run_board_computer().await;

            self.logs_cli_tx().send(LogEvent::System("L'ordinateur de bord s'est arrêté de façon innatendue. Veuillez le relancer manuellement.".red()));
        })
    }
}
