use std::sync::{Arc, RwLock, mpsc::channel};

use crate::{ais::AisRunner, antenna::{Antenna, Packet}, boat_info::BoatInfo, boats_registry::{self, BoatsInfoRegistry}, gps::{self, Gps}};

pub struct Boat {
    pub boat_info: Arc<BoatInfo>,
    pub ais: Arc<AisRunner>,
    pub gps: Arc<Gps>,
    pub boats_registry: Arc<RwLock<BoatsInfoRegistry>>,
    pub antenna_87_b: Arc<Antenna>,
    pub antenna_88_b: Arc<Antenna>,
    pub gps_antenna: Arc<Antenna>
}


impl Boat {
    pub fn init() -> Self {
        let (ais_tx, ais_rx) = channel::<Packet>();
        let (gps_tx, gps_rx) = channel::<String>();
        let (c_87_b_tx, c_87_b_rx) = channel::<String>();
        let (c_88_b_tx, c_88_b_rx) = channel::<String>();
        let (c_gps_tx, c_gps_rx) = channel::<String>();

        let ant1: Antenna = Antenna::init(Some(161975000), Some(ais_tx.clone()), None, c_87_b_tx.clone(), c_87_b_rx);
        let ant2: Antenna = Antenna::init(Some(161975001), Some(ais_tx.clone()), None, c_88_b_tx.clone(), c_88_b_rx);
        let ant3: Antenna = Antenna::init(None, None, Some(gps_tx.clone()), c_gps_tx.clone(), c_gps_rx);

        let boat_info: Arc<BoatInfo> = Arc::new(BoatInfo::init());
        let boats_registry: Arc<RwLock<BoatsInfoRegistry>> = Arc::new(RwLock::new(BoatsInfoRegistry::init()));

        let ais: AisRunner = AisRunner::init(ais_tx.clone(), ais_rx, c_87_b_tx.clone(), c_88_b_tx.clone(), Arc::clone(&boat_info), Arc::clone(&boats_registry));
        let gps: Gps = Gps::init(Arc::clone(&boat_info), gps_rx, gps_tx.clone(), c_gps_tx.clone());

        Self {
            boat_info: boat_info,
            ais: Arc::new(ais),
            gps: Arc::new(gps),
            boats_registry: boats_registry,
            antenna_87_b: Arc::new(ant1),
            antenna_88_b: Arc::new(ant2),
            gps_antenna: Arc::new(ant3)
        }
    }


    pub fn start(self) -> () {
        self.antenna_87_b.start();
        self.antenna_88_b.start();
        self.gps_antenna.start();
        self.gps.start();
        self.ais.start();
    }
}