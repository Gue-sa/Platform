use crate::{
    board_computer::BoardComputer, boat_ais::BoatAisRunner, boat_gps::BoatGps,
    navigator::Navigator, serial_driver::SerialDriver, systemstate::SystemState, ui::Ui,
};
use shared::{
    antenna::Antenna,
    boat_info::BoatInfo,
    common::{errors::BoatResult, types::LogEvent},
    config::Config,
    logs_cli::LogsCli,
    radio_builder::build_radio,
    satcom::SatCom,
};
use std::sync::{Arc, Mutex, mpsc::channel};
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
    logs_cli: LogsCli,
    system_state: Arc<SystemState>,
}

impl Boat {
    pub async fn init() -> BoatResult<Self> {
        let config = Config::load().unwrap();

        let boat_info_arc = Arc::new(BoatInfo::new(None, None, None));

        let (cli_tx, cli_rx) = channel::<LogEvent>();

        let cli = LogsCli::new(
            cli_rx,
            config.boat_sys_logs_filename(),
            config.boat_ais_logs_filename(),
            config.boat_gps_logs_filename(),
            config.boat_satcom_logs_filename(),
            config.boat_computer_logs_filename(),
            "Logs Bateau",
        );

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
            boats_reg_arc,
        ) = build_radio(cli_tx.clone(), *boat_info_arc.get_static_data()?.mmsi()).await?;

        let system_state_arc = Arc::new(SystemState::new(cli_tx.clone()));
        let voyage = None;

        let ui = Ui::init(boat_info_arc.clone());

        let ais = BoatAisRunner::init(
            ais_rx,
            c87b_tx,
            c88b_tx,
            Arc::clone(&boat_info_arc),
            boats_reg_arc.clone(),
            system_state_arc.clone(),
            cli_tx.clone(),
        )
        .unwrap();
        let gps = BoatGps::init(
            Arc::clone(&boat_info_arc),
            gps_rx,
            c_gps_tx,
            system_state_arc.clone(),
            cli_tx.clone(),
        );

        let serial_driver = SerialDriver::init(cli_tx.clone());
        let serial_driver_arc = Arc::new(Mutex::new(serial_driver));

        let navigator = Navigator::init(
            Arc::new(Mutex::new(None)),
            serial_driver_arc.clone(),
            boat_info_arc.clone(),
            cli_tx.clone(),
        );

        let board_computer = BoardComputer::init(
            boat_info_arc.clone(),
            boats_reg_arc,
            voyage,
            board_computer_rx,
            sender_satcom_tx,
            navigator,
            cli_tx.clone(),
        );

        Ok(Self {
            system_state: system_state_arc,
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
        let config = Config::load().unwrap();

        let _c87b_antenna_handle = self.c87b_antenna.start().await?;
        let _c88b_antenna_handle = self.c88b_antenna.start().await?;
        let _gps_antenna_handle = self.gps_antenna.start().await?;
        let _satcom_antenna_handle = self.satcom_antenna.start().await?;

        let _gps_handle: Option<JoinHandle<()>> = if *config.gps_detection() {
            Some(self.gps.start())
        } else {
            None
        };

        let _satcom_handle = self.satcom.start();
        let _board_computer_handle = self.board_computer.start();
        let _ais_handle = self.ais.start();

        let _logs_cli_handle = if *config.cli() {
            Some(self.logs_cli.run().unwrap())
        } else {
            None
        };

        if *config.gui() {
            self.ui.start();
        };

        Ok(())
    }
}
