use std::sync::Arc;

use shared::{
    boat_info::BoatInfo,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::HARBOURMASTER_MMSI,
        types::{SatComMessageType, VoyageStatus},
    },
    satcom_message::SatComMessage,
    voyage_order::{VoyageOrder, VoyageOrderBody, VoyageOrderHeader},
};
use tokio::sync::mpsc::{Receiver, Sender};

use colored::*;

use crate::{common::utils::log, voyage::Voyage};

pub struct BoardComputer {
    boat_info: Arc<BoatInfo>,
    boats_registry: Arc<BoatsInfoRegistry>,
    voyage: Option<Voyage>,
    rx: Receiver<SatComMessage>,
    satcom_tx: Sender<SatComMessage>,
    voyage_order_revision: Option<VoyageOrder>,
}

impl BoardComputer {
    pub fn new(
        boat_info: Arc<BoatInfo>,
        boats_registry: Arc<BoatsInfoRegistry>,
        voyage: Option<Voyage>,
        rx: Receiver<SatComMessage>,
        satcom_tx: Sender<SatComMessage>,
    ) -> Self {
        Self {
            boat_info: boat_info,
            boats_registry: boats_registry,
            voyage: voyage,
            rx: rx,
            satcom_tx: satcom_tx,
            voyage_order_revision: None,
        }
    }

    fn order_id(&self) -> u16 {
        *self.voyage.as_ref().unwrap().order().header().id()
    }

    fn has_voyage(&self) -> bool {
        self.voyage.is_some()
    }

    fn update_voyage(&mut self, new_voyage: Voyage) -> () {
        self.voyage = Some(new_voyage);
    }

    fn update_voyage_status(&mut self, status: VoyageStatus) -> () {
        if let Some(ref mut voyage) = self.voyage {
            voyage.set_status(status);
        }
    }

    fn matches_status(&self, status: Option<VoyageStatus>) -> bool {
        match (self.voyage.as_ref(), status) {
            (Some(v), Some(s)) => *v.status() == s,
            (None, None) => true,
            _ => false,
        }
    }

    fn adopt_voyage_order(&mut self, order: &VoyageOrder) -> () {
        self.update_voyage(Voyage::from(
            order.clone(),
            (
                *self.boat_info.get_navigation_data().longitude() as u16, // ATTENTION, CE N'EST PAS CORRECT, MAIS ON PART DU PRINCIPE QUE COMME ON UTILISER UN REPERE 1920x1080, UN u16 SUFFIT !
                *self.boat_info.get_navigation_data().latitude() as u16,
            ),
        ));

        let order_body: VoyageOrderBody = order.body();

        self.boat_info.update_voyage_data(
            Some(order_body.destination().to_string()),
            Some(*order_body.eta_month()),
            Some(*order_body.eta_day()),
            Some(*order_body.eta_hour()),
            Some(*order_body.eta_minute()),
        );
    }

    fn adopt_voyage_order_revision(&mut self) -> () {
        if let Some(order) = self.voyage_order_revision.take() {
            self.adopt_voyage_order(&order);
        }
    }

    async fn respond(
        &mut self,
        satcom_message: &SatComMessage,
        message_type: SatComMessageType,
        response_order_header: Option<VoyageOrderHeader>,
        response_order_revision: Option<VoyageOrderBody>,
    ) -> () {
        let message = SatComMessage::new(
            *self.boat_info.get_static_data().mmsi(),
            HARBOURMASTER_MMSI,
            response_order_header.unwrap_or(satcom_message.order_header()),
            message_type,
            response_order_revision,
        );

        self.satcom_tx.send(message).await;
    }

    async fn update_voyage_status_and_respond(
        &mut self,
        new_status: VoyageStatus,
        satcom_message: &SatComMessage,
        message_type: SatComMessageType,
        response_order_header: Option<VoyageOrderHeader>,
        response_order_revision: Option<VoyageOrderBody>,
        log_msg: String,
    ) -> () {
        self.update_voyage_status(new_status);

        self.respond(
            satcom_message,
            message_type,
            response_order_header,
            response_order_revision,
        )
        .await;

        log(log_msg.cyan());
    }

    async fn handle_offer(&mut self, satcom_message: &SatComMessage) -> () {
        self.respond(
            satcom_message,
            SatComMessageType::Acknowledgement,
            None,
            None,
        )
        .await;

        log(format!(
            "Offre d'ordre de voyage reçue (ID {}). Accusé de réception envoyé.",
            satcom_message.order_header().id()
        )
        .cyan());

        if !self.has_voyage() {
            let voyage_order: &VoyageOrder = &satcom_message.order().unwrap();

            self.adopt_voyage_order(voyage_order);

            self.update_voyage_status_and_respond(
                VoyageStatus::RevisionAccepted,
                satcom_message,
                SatComMessageType::RevisionAcceptation,
                None,
                None,
                format!(
                    "Ordre de voyage {} accepté.",
                    satcom_message.order_header().id()
                ),
            )
            .await;
        } else {
            self.respond(
                satcom_message,
                SatComMessageType::RevisionRefusal,
                None,
                None,
            )
            .await;

            log(format!(
                "Ordre de voyage {} refusé (navire déjà en activité).",
                satcom_message.order_header().id()
            )
            .cyan());
        }
    }

    async fn handle_revision_request_acknowledgement(
        &mut self,
        satcom_message: &SatComMessage,
    ) -> () {
        self.voyage_order_revision
            .as_mut()
            .unwrap()
            .set_version(*satcom_message.order_header().version());

        self.update_voyage_status(VoyageStatus::UnderRevision);

        log(format!(
            "Demande de révision de l'ordre {} reçu par la capitainerie. Attente d'une réponse.",
            satcom_message.order_header().id()
        )
        .cyan());
    }

    async fn handle_initial_revision_acceptation_acknowledgement(
        &mut self,
        satcom_message: &SatComMessage,
    ) -> () {
        self.update_voyage_status_and_respond(
            VoyageStatus::InExecution,
            satcom_message,
            SatComMessageType::ExecutingLastAgreedRevision,
            None,
            None,
            format!(
                "Ordre de voyage {} en cours d'exécution.",
                satcom_message.order_header().id()
            ),
        )
        .await;
    }

    async fn handle_revision_acceptation(&mut self, satcom_message: &SatComMessage) -> () {
        self.voyage_order_revision
            .as_mut()
            .unwrap()
            .set_version(*satcom_message.order_header().version());

        self.update_voyage_status_and_respond(
                VoyageStatus::RevisionAccepted,
                satcom_message,
                SatComMessageType::RevisionAcceptation,
                None,
                None,
                format!(
                    "Révision de l'ordre de voyage {} acceptée. Nouvelle version : n°{}. Exécution en cours.",
                    satcom_message.order_header().id(), satcom_message.order_header().version()
                )
            )
            .await;
    }

    async fn handle_revision_refusal(&mut self, satcom_message: &SatComMessage) -> () {
        self.update_voyage_status_and_respond(
                VoyageStatus::RevisionRefused,
                satcom_message,
                SatComMessageType::RevisionRefusal,
                None,
                None,
                format!(
                    "Révision de l'ordre de voyage {} refusée. Retour à la version n°{}. Exécution en cours.",
                    satcom_message.order_header().id(), satcom_message.order_header().version()
                )
            )
            .await;
    }

    async fn handle_revision_request(&mut self, satcom_message: &SatComMessage) -> () {
        self.voyage_order_revision = Some(satcom_message.order().unwrap());

        self.update_voyage_status_and_respond(
            VoyageStatus::UnderRevision,
            satcom_message,
            SatComMessageType::Acknowledgement,
            None,
            None,
            format!(
                "Demande de révision de l'ordre de voyage {} reçue. Traitement de la demande.",
                satcom_message.order_header().id()
            ),
        )
        .await;

        self.adopt_voyage_order_revision();

        self.update_voyage_status_and_respond(
                VoyageStatus::RevisionAccepted,
                satcom_message,
                SatComMessageType::RevisionAcceptation,
                None,
                None,
                format!(
                    "Révision de l'ordre de voyage {} acceptée. Nouvelle version : n°{}. Exécution en cours.",
                    satcom_message.order_header().id(), satcom_message.order_header().version()
                )
            )
            .await;
    }

    async fn handle_end_of_voyage(&mut self, satcom_message: &SatComMessage) -> () {
        self.update_voyage_status_and_respond(
            VoyageStatus::Finished,
            satcom_message,
            SatComMessageType::Acknowledgement,
            None,
            None,
            format!(
                "Ordre de voyage {} achevé. Attente d'un nouvel ordre.",
                satcom_message.order_header().id()
            ),
        )
        .await;
    }

    pub async fn start(mut self) -> () {
        // ATTENTION : tout ce qui touche à la révision d'ordres de voyage en cours de route est très hasardeux, pour ne pas dire 0% fonctionnel.
        tokio::spawn(async move {
            let my_mmsi: u32 = *self.boat_info.get_static_data().mmsi();
            while let Some(satcom_message) = self.rx.recv().await {
                if *satcom_message.target() != my_mmsi
                    || *satcom_message.source() != HARBOURMASTER_MMSI
                {
                    continue;
                }

                let concerns_current_voyage: bool = self.voyage.as_ref().map_or(false, |v| {
                    v.order().header() == satcom_message.order_header()
                });

                let concerns_current_revision: bool = self
                    .voyage_order_revision
                    .as_ref()
                    .map_or(false, |rev| rev.header() == satcom_message.order_header());

                match *satcom_message.message_type() {
                    SatComMessageType::Offer => {
                        self.handle_offer(&satcom_message).await;
                    }
                    SatComMessageType::Acknowledgement => {
                        if self.matches_status(Some(VoyageStatus::RevisionSubmitted))
                            && concerns_current_revision
                        {
                            self.handle_revision_request_acknowledgement(&satcom_message)
                                .await;
                        } else if self.matches_status(Some(VoyageStatus::RevisionAccepted))
                            && concerns_current_voyage
                        {
                            self.handle_initial_revision_acceptation_acknowledgement(
                                &satcom_message,
                            )
                            .await;
                        }
                    }
                    SatComMessageType::RevisionAcceptation => {
                        if self.matches_status(Some(VoyageStatus::RevisionSubmitted))
                            && concerns_current_revision
                        {
                            self.handle_revision_acceptation(&satcom_message).await;
                        }
                    }
                    SatComMessageType::RevisionRefusal => {
                        if self.matches_status(Some(VoyageStatus::RevisionSubmitted))
                            && concerns_current_revision
                        {
                            self.handle_revision_refusal(&satcom_message).await;
                        }
                    }
                    SatComMessageType::RevisionRequest => {
                        if (self.matches_status(Some(VoyageStatus::RevisionAccepted))
                            || self.matches_status(Some(VoyageStatus::RevisionRefused))
                            || self.matches_status(Some(VoyageStatus::InExecution)))
                            && self.order_id() == *satcom_message.order_header().id()
                        {
                            self.handle_revision_request(&satcom_message).await;
                        }
                    }
                    SatComMessageType::EndOfVoyage => {
                        if self.matches_status(Some(VoyageStatus::Completed))
                            && concerns_current_voyage
                        {
                            self.handle_end_of_voyage(&satcom_message).await;
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}
