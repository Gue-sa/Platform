use std::sync::{Arc, RwLock};

use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    shared::{
        boat_info::BoatInfo,
        common::types::{SatComMessageType, VoyageStatus},
        satcom_message::SatComMessage,
        voyage_order::VoyageOrder,
    },
    voyage::Voyage,
};

pub struct BoardComputer {
    pub boat_info: Arc<BoatInfo>,
    pub voyage: Arc<RwLock<Option<Voyage>>>,
    pub rx: Receiver<SatComMessage>,
    pub satcom_tx: Sender<SatComMessage>,
    pub voyage_order_review: Option<VoyageOrder>,
}

impl BoardComputer {
    pub fn new(
        boat_info: Arc<BoatInfo>,
        voyage: Arc<RwLock<Option<Voyage>>>,
        rx: Receiver<SatComMessage>,
        satcom_tx: Sender<SatComMessage>,
    ) -> Self {
        Self {
            boat_info: boat_info,
            voyage: voyage,
            rx: rx,
            satcom_tx: satcom_tx,
            voyage_order_review: None
        }
    }

    pub fn order_id(&self) -> u32 {
        self.voyage.read().unwrap().as_ref().unwrap().order.id
    }

    pub fn order_version(&self) -> u8 {
        self.voyage.read().unwrap().as_ref().unwrap().order.version
    }

    pub fn has_voyage(&self) -> bool {
        self.voyage.read().unwrap().is_some()
    }

    pub fn update_voyage(&self, new_voyage: Voyage) -> () {
        *self.voyage.write().unwrap() = Some(new_voyage);
    }

    pub fn update_voyage_status(&self, status: VoyageStatus) -> () {
        if let Some(ref mut voyage) = *self.voyage.write().unwrap() {
            voyage.set_status(status);
        }
    }

    pub fn voyage_status(&self) -> Option<VoyageStatus> {
        if let Some(ref voyage) = *self.voyage.read().unwrap() {
            Some(voyage.status.clone())
        } else {
            None
        }
    }

    pub fn matches_status(&self, status: Option<VoyageStatus>) -> bool {
        match status {
            Some(status_value) => {
                if let Some(ref voyage) = *self.voyage.read().unwrap() {
                    return voyage.status == status_value;
                } else {
                    false
                }
            }
            None => self.voyage.read().unwrap().is_none(),
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
            Some(order.destination),
            Some(order.eta_month),
            Some(order.eta_day),
            Some(order.eta_hour),
            Some(order.eta_minute),
        );
    }

    pub fn drop_voyage_order_revision(&mut self) -> () {
        self.voyage_order_review = None;
    }

    pub fn adopt_voyage_order_revision(&mut self) -> () {
        let order: VoyageOrder = self.voyage_order_review.as_ref().unwrap().clone();

        self.adopt_voyage_order(order);

        self.drop_voyage_order_revision();
    }

    pub fn end_voyage(&self) -> () {
        *self.voyage.write().unwrap() = None;
    }

    pub async fn start(mut self) -> () {
        loop {
            match self.rx.recv().await {
                Some(satcom_message) => {
                    if satcom_message.target == self.boat_info.get_static_data().mmsi {
                        let mut msg_template: SatComMessage = SatComMessage::new(
                            self.boat_info.get_static_data().mmsi,
                            satcom_message.source,
                            satcom_message.order_id,
                            satcom_message.order_version,
                            SatComMessageType::Acknowledgement,
                            None,
                        );

                        match satcom_message.message_type {
                            SatComMessageType::Offer => {
                                let _ = self.satcom_tx.send(msg_template.clone()).await;

                                if !self.has_voyage() {
                                    let voyage_order: VoyageOrder =
                                        satcom_message.order_review.unwrap();

                                    self.adopt_voyage_order(voyage_order);

                                    self.update_voyage_status(VoyageStatus::Accepted);
                                    msg_template.message_type = SatComMessageType::Acceptation;
                                    let _ = self.satcom_tx.send(msg_template).await;
                                } else {
                                    msg_template.message_type = SatComMessageType::Refusal;
                                    let _ = self.satcom_tx.send(msg_template).await;
                                }
                            }
                            SatComMessageType::Acknowledgement => {
                                if self.matches_status(Some(VoyageStatus::RevisionSubmitted)) {
                                    self.update_voyage_status(VoyageStatus::UnderRevision);
                                } else if self.matches_status(Some(VoyageStatus::Accepted)) {
                                    self.update_voyage_status(VoyageStatus::InExecution);
                                    msg_template.message_type = SatComMessageType::Executing;
                                    let _ = self.satcom_tx.send(msg_template).await;
                                }
                            }
                            SatComMessageType::Acceptation => {
                                if self.matches_status(Some(VoyageStatus::UnderRevision)) {
                                    self.adopt_voyage_order_revision();
                                    self.update_voyage_status(VoyageStatus::InExecution);
                                    msg_template.message_type = SatComMessageType::Executing;
                                    let _ = self.satcom_tx.send(msg_template).await;
                                }
                            }
                            SatComMessageType::Refusal => {
                                if self.matches_status(Some(VoyageStatus::UnderRevision)) {
                                    let _ = self.satcom_tx.send(msg_template.clone()).await;
                                    self.drop_voyage_order_revision();
                                    self.update_voyage_status(VoyageStatus::InExecution);
                                    msg_template.message_type = SatComMessageType::Executing;
                                    msg_template.order_version = self.order_version();
                                    let _ = self.satcom_tx.send(msg_template).await;
                                }
                            }
                            SatComMessageType::Revision => {
                                if self.matches_status(Some(VoyageStatus::Accepted))
                                    || self.matches_status(Some(VoyageStatus::InExecution))
                                {
                                    self.voyage_order_review = satcom_message.order_review.clone();
                                    self.update_voyage_status(VoyageStatus::UnderRevision);
                                    msg_template.order_version = satcom_message.order_review.unwrap().version;
                                    let _ = self.satcom_tx.send(msg_template.clone()).await;
                                    self.adopt_voyage_order_revision();
                                    self.update_voyage_status(VoyageStatus::Accepted);
                                    msg_template.message_type = SatComMessageType::Acceptation;
                                    let _ = self.satcom_tx.send(msg_template).await;
                                }
                            }
                            SatComMessageType::EndOfVoyage => {
                                if self.matches_status(Some(VoyageStatus::Completed)) {
                                    self.update_voyage_status(VoyageStatus::Finished);
                                    let _ = self.satcom_tx.send(msg_template.clone()).await;
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
