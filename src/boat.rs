use std::sync::Arc;

use tokio::sync::{Semaphore, RwLock, mpsc::*};

use crate::{ais::AisRunner, antenna::{Antenna, Packet}, boat_info::BoatInfo, boats_registry::BoatsInfoRegistry, common::bitpacker::BitPacker, gps::Gps};

pub struct Boat {
    boat_info: Arc<BoatInfo>,
    ais: AisRunner,
    gps: Gps,
    antenna_87_b: Antenna,
    antenna_88_b: Antenna,
    gps_antenna: Antenna
}


impl Boat {
    pub async fn init() -> Self {
        let (ais_tx, ais_rx) = channel::<Packet>(Semaphore::MAX_PERMITS);
        let (gps_tx, gps_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (c_87_b_tx, c_87_b_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (c_88_b_tx, c_88_b_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (c_gps_tx, c_gps_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);

        let ant1: Antenna = Antenna::init(Some(161975000), Some(ais_tx.clone()), None, c_87_b_tx.clone(), c_87_b_rx).await;
        let ant2: Antenna = Antenna::init(Some(161975001), Some(ais_tx.clone()), None, c_88_b_tx.clone(), c_88_b_rx).await;
        let ant3: Antenna = Antenna::init(None, None, Some(gps_tx.clone()), c_gps_tx.clone(), c_gps_rx).await;

        let boat_info: Arc<BoatInfo> = Arc::new(BoatInfo::init(None, None, None));
        let boats_registry: BoatsInfoRegistry = BoatsInfoRegistry::init();

        let ais: AisRunner = AisRunner::init(ais_tx.clone(), ais_rx, c_87_b_tx.clone(), c_88_b_tx.clone(), Arc::clone(&boat_info), boats_registry);
        let gps: Gps = Gps::init(Arc::clone(&boat_info), gps_rx, gps_tx.clone(), c_gps_tx.clone());

        Self {
            boat_info: boat_info,
            ais: ais,
            gps: gps,
            antenna_87_b: ant1,
            antenna_88_b: ant2,
            gps_antenna: ant3
        }
    }


    pub fn info(&self) -> Arc<BoatInfo> {
        self.boat_info.clone()
    }


    pub async fn start(self) -> () {
        self.antenna_87_b.start().await;
        self.antenna_88_b.start().await;
        self.gps_antenna.start().await;
        self.gps.start().await;
        self.ais.start().await;
    }
}