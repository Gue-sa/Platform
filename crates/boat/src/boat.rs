use std::sync::Arc;

use shared::{
    antenna::Antenna,
    bitpacker::BitPacker,
    boat_info::BoatInfo,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::{
            C87B_EM_PORT, C87B_REC_PORT, C88B_EM_PORT, C88B_REC_PORT, GPS_EM_PORT, GPS_REC_PORT,
            SATCOM_EM_PORT, SATCOM_REC_PORT,
        },
        types::{AisPacket, Channel},
    },
    satcom::SatCom,
    satcom_message::SatComMessage,
};
use tokio::sync::{Semaphore, mpsc::*};

use crate::{
    board_computer::BoardComputer, boat_ais::BoatAisRunner, boat_gps::BoatGps,
    systemstate::SystemState, ui::Ui, voyage::Voyage,
};

pub struct Boat {
    pub ais: BoatAisRunner,
    pub gps: BoatGps,
    pub satcom: SatCom,
    pub board_computer: BoardComputer,
    pub antenna_87_b: Antenna,
    pub antenna_88_b: Antenna,
    pub gps_antenna: Antenna,
    pub satcom_antenna: Antenna,
    pub ui: Ui,
    pub system_state: Arc<SystemState>,
}

impl Boat {
    pub async fn init() -> Self {
        let (ais_tx, ais_rx) = channel::<AisPacket>(Semaphore::MAX_PERMITS);
        let (gps_tx, gps_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (sender_satcom_tx, sender_satcom_rx) = channel::<SatComMessage>(Semaphore::MAX_PERMITS);
        let (reader_satcom_tx, reader_satcom_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (board_computer_tx, board_computer_rx) =
            channel::<SatComMessage>(Semaphore::MAX_PERMITS);

        let (c_87_b_tx, c_87_b_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (c_88_b_tx, c_88_b_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (c_gps_tx, c_gps_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (c_satcom_tx, c_satcom_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);

        let ant1: Antenna = Antenna::init(
            Some(ais_tx.clone()),
            None,
            None,
            c_87_b_rx,
            C87B_REC_PORT,
            C87B_EM_PORT,
            Channel::C87B,
        )
        .await;
        let ant2: Antenna = Antenna::init(
            Some(ais_tx),
            None,
            None,
            c_88_b_rx,
            C88B_REC_PORT,
            C88B_EM_PORT,
            Channel::C88B,
        )
        .await;
        let ant3: Antenna = Antenna::init(
            None,
            Some(gps_tx),
            None,
            c_gps_rx,
            GPS_REC_PORT,
            GPS_EM_PORT,
            Channel::GPS,
        )
        .await;
        let ant4: Antenna = Antenna::init(
            None,
            None,
            Some(reader_satcom_tx),
            c_satcom_rx,
            SATCOM_REC_PORT,
            SATCOM_EM_PORT,
            Channel::SATCOM,
        )
        .await;

        let boat_info: Arc<BoatInfo> = Arc::new(BoatInfo::init(None, None, None));
        let boats_registry: Arc<BoatsInfoRegistry> = Arc::new(BoatsInfoRegistry::init());

        let system_state: Arc<SystemState> = Arc::new(SystemState::new());

        let ais: BoatAisRunner = BoatAisRunner::init(
            ais_rx,
            c_87_b_tx,
            c_88_b_tx,
            Arc::clone(&boat_info),
            boats_registry.clone(),
            system_state.clone(),
        );
        let gps: BoatGps = BoatGps::init(
            Arc::clone(&boat_info),
            gps_rx,
            c_gps_tx,
            system_state.clone(),
        );
        let satcom: SatCom = SatCom::new(
            reader_satcom_rx,
            sender_satcom_rx,
            c_satcom_tx,
            board_computer_tx,
        );
        let voyage: Option<Voyage> = None;
        let board_computer = BoardComputer::new(
            boat_info.clone(),
            boats_registry,
            voyage,
            board_computer_rx,
            sender_satcom_tx,
        );

        let ui: Ui = Ui::init(boat_info.clone());

        Self {
            system_state: system_state,
            antenna_87_b: ant1,
            antenna_88_b: ant2,
            gps_antenna: ant3,
            satcom_antenna: ant4,
            ais: ais,
            gps: gps,
            satcom: satcom,
            board_computer: board_computer,
            ui: ui,
        }
    }

    pub async fn start(self) -> () {
        tokio::spawn(async move {
            self.antenna_87_b.start().await;
            self.antenna_88_b.start().await;
            self.gps_antenna.start().await;
            self.satcom_antenna.start().await;
            self.gps.start().await;
            self.satcom.start().await;
            self.board_computer.start().await;
            self.ais.start().await;
        });
        
        self.ui.start();
    }
}
