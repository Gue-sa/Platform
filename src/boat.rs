use std::sync::{Arc, RwLock};

use tokio::sync::{Semaphore, mpsc::*};

use crate::{
    board_computer::{self, BoardComputer},
    boat_ais::AisRunner,
    boat_antenna::{AisPacket, Antenna},
    boat_gps::Gps,
    satcom::SatCom,
    shared::{
        bitpacker::BitPacker,
        boat_info::BoatInfo,
        boats_registry::BoatsInfoRegistry,
        common::{
            constants::{
                C87B_EM_PORT, C87B_REC_PORT, C88B_EM_PORT, C88B_REC_PORT, GPS_EM_PORT,
                GPS_REC_PORT, SATCOM_EM_PORT, SATCOM_REC_PORT,
            },
            types::*,
        },
        satcom_message::SatComMessage,
    },
    systemstate::SystemState,
    ui::Ui,
    voyage::Voyage,
};

pub struct Boat {
    pub boat_info: Arc<BoatInfo>,
    pub ais: AisRunner,
    pub gps: Gps,
    pub satcom: SatCom,
    pub board_computer: BoardComputer,
    pub antenna_87_b: Antenna,
    pub antenna_88_b: Antenna,
    pub gps_antenna: Antenna,
    pub satcom_antenna: Antenna,
    pub ui: Ui,
    pub system_state: Arc<SystemState>,
    pub voyage: Arc<RwLock<Option<Voyage>>>,
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
            Some(ais_tx.clone()),
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
            Some(gps_tx.clone()),
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
            Some(reader_satcom_tx.clone()),
            c_satcom_rx,
            SATCOM_REC_PORT,
            SATCOM_EM_PORT,
            Channel::SATCOM,
        )
        .await;

        let boat_info: Arc<BoatInfo> = Arc::new(BoatInfo::init(None, None, None));
        let boats_registry: BoatsInfoRegistry = BoatsInfoRegistry::init();

        let system_state: Arc<SystemState> = Arc::new(SystemState::new());

        let ais: AisRunner = AisRunner::init(
            ais_rx,
            c_87_b_tx.clone(),
            c_88_b_tx.clone(),
            Arc::clone(&boat_info),
            boats_registry,
            system_state.clone(),
        );
        let gps: Gps = Gps::init(
            Arc::clone(&boat_info),
            gps_rx,
            c_gps_tx.clone(),
            system_state.clone(),
        );
        let satcom: SatCom = SatCom::new(
            reader_satcom_rx,
            sender_satcom_rx,
            c_satcom_tx.clone(),
            board_computer_tx.clone(),
        );
        let voyage: Arc<RwLock<Option<Voyage>>> = Arc::new(RwLock::new(None));
        let board_computer = BoardComputer::new(
            boat_info.clone(),
            voyage.clone(),
            board_computer_rx,
            sender_satcom_tx.clone(),
        );

        let ui: Ui = Ui::init(boat_info.clone());

        Self {
            boat_info: boat_info,
            ais: ais,
            gps: gps,
            satcom: satcom,
            board_computer: board_computer,
            antenna_87_b: ant1,
            antenna_88_b: ant2,
            gps_antenna: ant3,
            satcom_antenna: ant4,
            ui: ui,
            system_state: system_state,
            voyage: voyage,
        }
    }

    pub async fn start(self) -> () {
        let _ = tokio::spawn(async move {
            self.antenna_87_b.start().await;
            self.antenna_88_b.start().await;
            self.gps_antenna.start().await;
            self.satcom_antenna.start().await;
            self.gps.start().await;
            self.ais.start().await;
            self.satcom.start().await;
            self.board_computer.start().await;
        });

        self.ui.start();
    }
}
