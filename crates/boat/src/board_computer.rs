use std::sync::Arc;

use shared::{
    boat_info::BoatInfo,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::HARBOURMASTER_MMSI,
        types::{SatComMessageType, VoyageStatus},
    },
    satcom_message::SatComMessage,
    voyage_order::VoyageOrder,
};
use tokio::sync::mpsc::{Receiver, Sender};

use colored::*;

use crate::{common::utils::log, voyage::Voyage};

pub struct BoardComputer {
    pub boat_info: Arc<BoatInfo>,
    pub boats_registry: Arc<BoatsInfoRegistry>,
    pub voyage: Option<Voyage>,
    pub rx: Receiver<SatComMessage>,
    pub satcom_tx: Sender<SatComMessage>,
    pub voyage_order_revision: Option<VoyageOrder>,
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

    pub fn order_id(&self) -> u16 {
        self.voyage.as_ref().unwrap().order.header.id
    }

    pub fn order_version(&self) -> u8 {
        self.voyage.as_ref().unwrap().order.header.version
    }

    pub fn has_voyage(&self) -> bool {
        self.voyage.is_some()
    }

    pub fn update_voyage(&mut self, new_voyage: Voyage) -> () {
        self.voyage = Some(new_voyage);
    }

    pub fn update_voyage_status(&mut self, status: VoyageStatus) -> () {
        if let Some(ref mut voyage) = self.voyage {
            voyage.set_status(status);
        }
    }

    pub fn voyage_status(&self) -> Option<VoyageStatus> {
        if let Some(ref voyage) = self.voyage {
            Some(voyage.status.clone())
        } else {
            None
        }
    }

    pub fn matches_status(&self, status: Option<VoyageStatus>) -> bool {
        match status {
            Some(status_value) => {
                if let Some(ref voyage) = self.voyage {
                    return voyage.status == status_value;
                } else {
                    false
                }
            }
            None => self.voyage.is_none(),
        }
    }

    pub fn adopt_voyage_order(&mut self, order: VoyageOrder) -> () {
        self.update_voyage(Voyage::from(
            order.clone(),
            (
                self.boat_info.get_navigation_data().longitude as u16, // ATTENTION, CE N'EST PAS CORRECT, MAIS ON PART DU PRINCIPE QUE COMME ON UTILISER UN REPERE 1920x1080, UN u16 SUFFIT !
                self.boat_info.get_navigation_data().latitude as u16,
            ),
        ));

        self.boat_info.update_voyage_data(
            Some(order.body.destination),
            Some(order.body.eta_month),
            Some(order.body.eta_day),
            Some(order.body.eta_hour),
            Some(order.body.eta_minute),
        );
    }

    pub fn drop_voyage_order_revision(&mut self) -> () {
        self.voyage_order_revision = None;
    }

    pub fn adopt_voyage_order_revision(&mut self) -> () {
        let order: VoyageOrder = self.voyage_order_revision.as_ref().unwrap().clone();

        self.adopt_voyage_order(order);

        self.drop_voyage_order_revision();
    }

    pub fn end_voyage(&mut self) -> () {
        self.voyage = None;
    }

    pub async fn start(mut self) -> () {
        // ATTENTION : tout ce qui touche à la révision d'ordres de voyage en cours de route est très hasardeux, pour ne pas dire 0% fonctionnel.
        tokio::spawn(async move {
            loop {
                match self.rx.recv().await {
                    Some(satcom_message) => {
                        if satcom_message.target == self.boat_info.get_static_data().mmsi
                            && satcom_message.source == HARBOURMASTER_MMSI
                        {
                            let mut msg_template: SatComMessage = SatComMessage::new(
                                self.boat_info.get_static_data().mmsi,
                                HARBOURMASTER_MMSI,
                                satcom_message.order_header.clone(),
                                SatComMessageType::Acknowledgement,
                                None,
                            );

                            match satcom_message.message_type {
                                SatComMessageType::Offer => {
                                    self.satcom_tx.send(msg_template.clone()).await;

                                    log(format!(
                                        "Offre d'ordre de voyage reçue (ID {}). Accusé de réception envoyé.",
                                        satcom_message.clone().order_header.id
                                    )
                                    .cyan());

                                    if !self.has_voyage() {
                                        let voyage_order: VoyageOrder = VoyageOrder {
                                            header: satcom_message.clone().order_header,
                                            body: satcom_message.clone().order_body_review.unwrap(),
                                        };

                                        self.adopt_voyage_order(voyage_order);

                                        self.update_voyage_status(VoyageStatus::RevisionAccepted);

                                        msg_template.message_type =
                                            SatComMessageType::RevisionAcceptation;
                                        self.satcom_tx.send(msg_template).await;

                                        log(format!(
                                            "Ordre de voyage {} accepté.",
                                            satcom_message.clone().order_header.id
                                        )
                                        .cyan());
                                    } else {
                                        msg_template.message_type =
                                            SatComMessageType::RevisionRefusal;
                                        self.satcom_tx.send(msg_template).await;

                                        log(format!(
                                            "Ordre de voyage {} refusé (navire déjà en activité).",
                                            satcom_message.clone().order_header.id
                                        )
                                        .cyan());
                                    }
                                }
                                SatComMessageType::Acknowledgement => {
                                    if self.matches_status(Some(VoyageStatus::RevisionSubmitted))
                                        && satcom_message.order_header
                                            == self.voyage_order_revision.as_ref().unwrap().header
                                    {
                                        self.voyage_order_revision
                                            .as_mut()
                                            .unwrap()
                                            .header
                                            .version = satcom_message.order_header.version;

                                        self.update_voyage_status(VoyageStatus::UnderRevision);

                                        log(format!(
                                            "Demande de révision de l'ordre {} reçu par la capitainerie. Attente d'une réponse.",
                                            satcom_message.clone().order_header.id
                                        )
                                        .cyan());
                                    } else if self
                                        .matches_status(Some(VoyageStatus::RevisionAccepted))
                                        && satcom_message.order_header
                                            == self.voyage.as_ref().unwrap().order.header
                                    {
                                        self.update_voyage_status(VoyageStatus::InExecution);

                                        msg_template.message_type =
                                            SatComMessageType::ExecutingLastAgreedRevision;
                                        self.satcom_tx.send(msg_template).await;

                                        log(format!(
                                            "Exécution en cours de l'ordre {}.",
                                            satcom_message.clone().order_header.id
                                        )
                                        .cyan());
                                    } else if self
                                        .matches_status(Some(VoyageStatus::RevisionRefused))
                                        && satcom_message.order_header
                                            == self.voyage.as_ref().unwrap().order.header
                                    {
                                        self.voyage = None;
                                    }
                                }
                                SatComMessageType::RevisionAcceptation => {
                                    if self.matches_status(Some(VoyageStatus::UnderRevision))
                                        && satcom_message.order_header
                                            == self.voyage_order_revision.as_ref().unwrap().header
                                    {
                                        self.adopt_voyage_order_revision();

                                        self.update_voyage_status(VoyageStatus::InExecution);

                                        msg_template.message_type =
                                            SatComMessageType::ExecutingLastAgreedRevision;
                                        self.satcom_tx.send(msg_template).await;

                                        log(format!(
                                            "Révision de l'ordre de voyage {} acceptée. Nouvelle version : n°{}. Exécution en cours.",
                                            satcom_message.clone().order_header.id, satcom_message.clone().order_header.version
                                        )
                                        .cyan());
                                    }
                                }
                                SatComMessageType::RevisionRefusal => {
                                    if self.matches_status(Some(VoyageStatus::UnderRevision))
                                        && satcom_message.order_header
                                            == self.voyage_order_revision.as_ref().unwrap().header
                                    {
                                        self.satcom_tx.send(msg_template.clone()).await;

                                        self.drop_voyage_order_revision();

                                        self.update_voyage_status(VoyageStatus::InExecution);

                                        msg_template.message_type =
                                            SatComMessageType::ExecutingLastAgreedRevision;
                                        msg_template.order_header.version = self.order_version();
                                        self.satcom_tx.send(msg_template).await;

                                        log(format!(
                                            "Révision de l'ordre de voyage {} refusée. Retour à la version n°{}. Exécution en cours.",
                                            satcom_message.clone().order_header.id, satcom_message.clone().order_header.version
                                        )
                                        .cyan());
                                    }
                                }
                                SatComMessageType::RevisionRequest => {
                                    if (self.matches_status(Some(VoyageStatus::RevisionAccepted))
                                        || self.matches_status(Some(VoyageStatus::RevisionRefused))
                                        || self.matches_status(Some(VoyageStatus::InExecution)))
                                        && self.voyage.as_ref().unwrap().order.header.id
                                            == satcom_message.order_header.id
                                    {
                                        self.voyage_order_revision = Some(VoyageOrder {
                                            header: satcom_message.order_header.clone(),
                                            body: satcom_message.order_body_review.clone().unwrap(),
                                        });

                                        self.update_voyage_status(VoyageStatus::UnderRevision);

                                        log(format!(
                                            "Demande de révision de l'ordre de voyage {} reçue. Traitement de la demande.",
                                            satcom_message.clone().order_header.id
                                        )
                                        .cyan());

                                        msg_template.order_header = satcom_message.clone().order_header;
                                        self.satcom_tx.send(msg_template.clone()).await;

                                        self.adopt_voyage_order_revision();

                                        self.update_voyage_status(VoyageStatus::RevisionAccepted);

                                        msg_template.message_type =
                                            SatComMessageType::RevisionAcceptation;
                                        self.satcom_tx.send(msg_template).await;

                                        log(format!(
                                            "Révision de l'ordre de voyage {} acceptée. Nouvelle version : n°{}. Exécution en cours.",
                                            satcom_message.clone().order_header.id, satcom_message.clone().order_header.version
                                        )
                                        .cyan());
                                    }
                                }
                                SatComMessageType::EndOfVoyage => {
                                    if self.matches_status(Some(VoyageStatus::Completed))
                                        && satcom_message.order_header
                                            == self.voyage.as_ref().unwrap().order.header
                                    {
                                        self.update_voyage_status(VoyageStatus::Finished);

                                        self.satcom_tx.send(msg_template.clone()).await;

                                        log(format!(
                                            "Ordre de voyage {} achevé. Attente d'un nouvel ordre.",
                                            satcom_message.clone().order_header.id
                                        )
                                        .cyan());
                                    }
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
