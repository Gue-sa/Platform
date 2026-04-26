use crate::{
    board_computer::BoardComputer, boat_ais::BoatAisRunner, boat_gps::BoatGps,
    systemstate::SystemState, ui::Ui, voyage::Voyage,
};
use shared::{antenna::Antenna, boat_info::BoatInfo, radio_builder::build_radio, satcom::SatCom};
use std::sync::Arc;

pub struct Boat {
    ais: BoatAisRunner,
    gps: BoatGps,
    satcom: SatCom,
    board_computer: BoardComputer,
    c87b_antenna: Antenna,
    c88b_antenna: Antenna,
    gps_antenna: Antenna,
    satcom_antenna: Antenna,
    ui: Ui,
    system_state: Arc<SystemState>,
}

impl Boat {
    pub async fn init() -> Self {
        let (
            ais_rx,
            gps_rx,
            board_computer_rx,
            c87b_tx,
            c88b_tx,
            c_gps_tx,
            sender_satcom_tx,
            ant1,
            ant2,
            ant3,
            ant4,
            satcom,
            boats_reg,
        ) = build_radio().await;

        let boat_info: Arc<BoatInfo> = Arc::new(BoatInfo::new(None, None, None));
        let system_state: Arc<SystemState> = Arc::new(SystemState::new());
        let voyage: Option<Voyage> = None;

        let ui: Ui = Ui::init(boat_info.clone());

        let ais: BoatAisRunner = BoatAisRunner::init(
            ais_rx,
            c87b_tx,
            c88b_tx,
            Arc::clone(&boat_info),
            boats_reg.clone(),
            system_state.clone(),
        );
        let gps: BoatGps = BoatGps::init(
            Arc::clone(&boat_info),
            gps_rx,
            c_gps_tx,
            system_state.clone(),
        );
        let board_computer = BoardComputer::new(
            boat_info.clone(),
            boats_reg,
            voyage,
            board_computer_rx,
            sender_satcom_tx,
        );

        Self {
            system_state: system_state,
            c87b_antenna: ant1,
            c88b_antenna: ant2,
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
        self.c87b_antenna.start().await;
        self.c88b_antenna.start().await;
        self.gps_antenna.start().await;
        self.satcom_antenna.start().await;
        self.gps.start().await;
        self.satcom.start().await;
        self.board_computer.start().await;
        self.ais.start().await;

        self.ui.start();
    }
}
