use crate::{
    board_computer::BoardComputer, boat_ais::BoatAisRunner, boat_gps::BoatGps,
    boat_logs_cli::BoatLogsCli, systemstate::SystemState, ui::Ui, voyage::Voyage,
};
use shared::{
    antenna::Antenna,
    boat_info::BoatInfo,
    common::{errors::BoatResult, types::LogEvent},
    radio_builder::build_radio,
    satcom::SatCom,
};
use std::sync::{Arc, mpsc::channel};
use tokio::task::JoinHandle;

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
    logs_cli: BoatLogsCli,
    system_state: Arc<SystemState>,
}

impl Boat {
    pub async fn init() -> BoatResult<Self> {
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
        ) = build_radio().await?;

        let (cli_tx, cli_rx) = channel::<LogEvent>();

        let cli: BoatLogsCli = BoatLogsCli::new(cli_rx);

        let boat_info: Arc<BoatInfo> = Arc::new(BoatInfo::new(None, None, None));
        let system_state: Arc<SystemState> = Arc::new(SystemState::new(cli_tx.clone()));
        let voyage: Option<Voyage> = None;

        let ui: Ui = Ui::init(boat_info.clone());

        let ais: BoatAisRunner = BoatAisRunner::init(
            ais_rx,
            c87b_tx,
            c88b_tx,
            Arc::clone(&boat_info),
            boats_reg.clone(),
            system_state.clone(),
            cli_tx.clone(),
        )
        .unwrap();
        let gps: BoatGps = BoatGps::init(
            Arc::clone(&boat_info),
            gps_rx,
            c_gps_tx,
            system_state.clone(),
            cli_tx.clone(),
        );
        let board_computer = BoardComputer::init(
            boat_info.clone(),
            boats_reg,
            voyage,
            board_computer_rx,
            sender_satcom_tx,
            cli_tx.clone(),
        );

        Ok(Self {
            system_state: system_state,
            c87b_antenna: ant1,
            c88b_antenna: ant2,
            gps_antenna: ant3,
            satcom_antenna: ant4,
            ais: ais,
            gps: gps,
            satcom: satcom,
            board_computer: board_computer,
            logs_cli: cli,
            ui: ui,
        })
    }

    pub async fn start(self) -> BoatResult<()> {
        let _c87b_antenna_handle: JoinHandle<()> = self.c87b_antenna.start().await?;
        let _c88b_antenna_handle: JoinHandle<()> = self.c88b_antenna.start().await?;
        let _gps_antenna_handle: JoinHandle<()> = self.gps_antenna.start().await?;
        let _satcom_antenna_handle: JoinHandle<()> = self.satcom_antenna.start().await?;

        let _gps_handle: JoinHandle<()> = self.gps.start();
        let _satcom_handle: JoinHandle<()> = self.satcom.start();
        let _board_computer_handle: JoinHandle<()> = self.board_computer.start();
        let _ais_handle: (
            JoinHandle<()>,
            JoinHandle<()>,
            JoinHandle<()>,
            JoinHandle<()>,
        ) = self.ais.start();

        let _logs_cli_handle: Result<JoinHandle<()>, std::io::Error> = self.logs_cli.run();

        self.ui.start();

        Ok(())
    }
}
