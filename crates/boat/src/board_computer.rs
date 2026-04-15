use std::sync::{Arc, RwLock};

use shared::{
    boat_info::BoatInfo,
    boats_registry::BoatsInfoRegistry,
    common::types::{SatComMessageType, VoyageStatus},
    satcom_message::SatComMessage,
    voyage_order::{VoyageOrder, VoyageOrderBody},
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::voyage::Voyage;

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

    pub fn order_id(&self) -> u32 {
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
                self.boat_info.get_navigation_data().longitude,
                self.boat_info.get_navigation_data().latitude,
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
        loop {
            match self.rx.recv().await {
                Some(satcom_message) => {
                    if satcom_message.target == self.boat_info.get_static_data().mmsi {
                        let mut msg_template: SatComMessage = SatComMessage::new(
                            self.boat_info.get_static_data().mmsi,
                            satcom_message.source,
                            satcom_message.order_header.clone(),
                            SatComMessageType::Acknowledgement,
                            None,
                        );

                        match satcom_message.message_type {
                            SatComMessageType::Offer => {
                                self.satcom_tx.send(msg_template.clone()).await;

                                if !self.has_voyage() {
                                    let voyage_order: VoyageOrder = VoyageOrder {
                                        header: satcom_message.order_header,
                                        body: satcom_message.order_body_review.unwrap(),
                                    };

                                    self.adopt_voyage_order(voyage_order);

                                    self.update_voyage_status(VoyageStatus::Accepted);

                                    msg_template.message_type = SatComMessageType::Acceptation;
                                    self.satcom_tx.send(msg_template).await;
                                } else {
                                    msg_template.message_type = SatComMessageType::Refusal;
                                    self.satcom_tx.send(msg_template).await;
                                }
                            }
                            SatComMessageType::Acknowledgement => {
                                if self.matches_status(Some(VoyageStatus::RevisionSubmitted))
                                    && satcom_message.order_header
                                        == self.voyage_order_revision.clone().unwrap().header
                                {
                                    self.update_voyage_status(VoyageStatus::UnderRevision);
                                } else if self.matches_status(Some(VoyageStatus::Accepted))
                                    && satcom_message.order_header
                                        == self.voyage_order_revision.clone().unwrap().header
                                {
                                    self.update_voyage_status(VoyageStatus::InExecution);

                                    msg_template.message_type = SatComMessageType::Executing;
                                    self.satcom_tx.send(msg_template).await;
                                }
                            }
                            SatComMessageType::Acceptation => {
                                if self.matches_status(Some(VoyageStatus::UnderRevision))
                                    && satcom_message.order_header
                                        == self.voyage_order_revision.clone().unwrap().header
                                {
                                    self.adopt_voyage_order_revision();

                                    self.update_voyage_status(VoyageStatus::InExecution);

                                    msg_template.message_type = SatComMessageType::Executing;
                                    self.satcom_tx.send(msg_template).await;
                                }
                            }
                            SatComMessageType::Refusal => {
                                if self.matches_status(Some(VoyageStatus::UnderRevision))
                                    && satcom_message.order_header
                                        == self.voyage_order_revision.clone().unwrap().header
                                {
                                    self.satcom_tx.send(msg_template.clone()).await;

                                    self.drop_voyage_order_revision();

                                    self.update_voyage_status(VoyageStatus::InExecution);

                                    msg_template.message_type = SatComMessageType::Executing;
                                    msg_template.order_header.version = self.order_version();
                                    self.satcom_tx.send(msg_template).await;
                                }
                            }
                            SatComMessageType::Revision => {
                                if (self.matches_status(Some(VoyageStatus::Accepted))
                                    || self.matches_status(Some(VoyageStatus::InExecution)))
                                    && satcom_message.order_header
                                        == self.voyage_order_revision.clone().unwrap().header
                                {
                                    self.voyage_order_revision = Some(VoyageOrder {
                                        header: satcom_message.order_header.clone(),
                                        body: satcom_message.order_body_review.clone().unwrap(),
                                    });

                                    self.update_voyage_status(VoyageStatus::UnderRevision);

                                    msg_template.order_header = satcom_message.order_header;
                                    self.satcom_tx.send(msg_template.clone()).await;

                                    self.adopt_voyage_order_revision();

                                    self.update_voyage_status(VoyageStatus::Accepted);

                                    msg_template.message_type = SatComMessageType::Acceptation;
                                    self.satcom_tx.send(msg_template).await;
                                }
                            }
                            SatComMessageType::EndOfVoyage => {
                                if self.matches_status(Some(VoyageStatus::Completed))
                                    && satcom_message.order_header
                                        == self.voyage_order_revision.clone().unwrap().header
                                {
                                    self.update_voyage_status(VoyageStatus::Finished);

                                    self.satcom_tx.send(msg_template.clone()).await;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                None => {}
            }
        }
    }
}
