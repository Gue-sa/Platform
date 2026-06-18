use colored::Colorize;
use opencv::{
    core::{Point, Point2f, Scalar, Vector},
    highgui,
    imgproc::{FONT_HERSHEY_SIMPLEX, LINE_8, put_text},
    objdetect::{
        self, ArucoDetector, DetectorParameters, PredefinedDictionaryType, RefineParameters,
        draw_detected_markers, get_predefined_dictionary,
    },
    prelude::*,
    videoio::{
        self, CAP_PROP_BUFFERSIZE, CAP_PROP_FOURCC, CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH,
        CAP_V4L2, VideoCapture, VideoWriter,
    },
};
use shared::{bitpacker::BitPacker, common::types::LogEvent, config::Config};
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use tokio::{
    sync::{
        Mutex, MutexGuard, Semaphore,
        mpsc::{Receiver, Sender, channel},
    },
    task::JoinHandle,
};

pub struct HarbourmasterGps {
    rx: Mutex<Receiver<BitPacker>>,
    pos_rx: Mutex<Receiver<[u32; 3]>>,
    pos_tx: Sender<[u32; 3]>,
    antenna_tx: Sender<BitPacker>,
    latitude: AtomicU32,
    longitude: AtomicU32,
    heading: AtomicU32,
    logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
}

impl HarbourmasterGps {
    pub async fn init(
        rx: Receiver<BitPacker>,
        ant_tx: Sender<BitPacker>,
        cli_tx: std::sync::mpsc::Sender<LogEvent>,
    ) -> Self {
        let (pos_tx, pos_rx) = channel::<[u32; 3]>(Semaphore::MAX_PERMITS);

        Self {
            rx: Mutex::new(rx),
            pos_rx: Mutex::new(pos_rx),
            pos_tx: pos_tx,
            antenna_tx: ant_tx,
            latitude: AtomicU32::new(0),
            longitude: AtomicU32::new(0),
            heading: AtomicU32::new(0),
            logs_cli_tx: cli_tx,
        }
    }

    fn logs_cli_tx(&self) -> std::sync::mpsc::Sender<LogEvent> {
        self.logs_cli_tx.clone()
    }

    fn coordinates(&self) -> [u32; 2] {
        [
            self.latitude.load(Ordering::Relaxed),
            self.longitude.load(Ordering::Relaxed),
        ]
    }

    fn set_coordinates(&self, lat: u32, lon: u32, heading: u32) {
        self.latitude.store(lat, Ordering::Relaxed);
        self.longitude.store(lon, Ordering::Relaxed);
        self.heading.store(heading, Ordering::Relaxed);
    }

    async fn run_detect_and_send(&self) {
        self.logs_cli_tx().send(LogEvent::System(
            "Lancement du satellite GPS (ArUco)...".yellow(),
        ));

        let mut cam = VideoCapture::new(
            Config::load().unwrap().gps_cam_idx().unwrap().into(),
            CAP_V4L2,
        )
        .unwrap();

        cam.set(videoio::CAP_PROP_AUTO_EXPOSURE, 1.0);

        let fourcc = VideoWriter::fourcc('M', 'J', 'P', 'G').unwrap();
        cam.set(CAP_PROP_FOURCC, fourcc.into()).unwrap();

        cam.set(CAP_PROP_FRAME_WIDTH, 1920.).unwrap();
        cam.set(CAP_PROP_FRAME_HEIGHT, 1080.).unwrap();
        cam.set(CAP_PROP_BUFFERSIZE, 1.).unwrap();

        let mut frame = Mat::default();
        let mut flipped_frame = Mat::default();

        let dictionary = get_predefined_dictionary(PredefinedDictionaryType::DICT_4X4_50).unwrap();
        let mut params = DetectorParameters::default().unwrap();
        params.set_corner_refinement_method(objdetect::CORNER_REFINE_SUBPIX);
        let refine_params = RefineParameters::new(10., 3., true).unwrap();
        let mut detector = ArucoDetector::new(&dictionary, &params, refine_params).unwrap();

        let mut corners = Vector::<Vector<Point2f>>::new();
        let mut ids = Vector::<i32>::new();
        let mut rejected = Vector::<Vector<Point2f>>::new();

        loop {
            cam.read(&mut frame).unwrap();
            if frame.empty() {
                break;
            }

            detector
                .detect_markers(&frame, &mut corners, &mut ids, &mut rejected)
                .unwrap();

            if !ids.is_empty() {
                draw_detected_markers(
                    &mut frame,
                    &corners,
                    &ids,
                    Scalar::new(0.0, 255.0, 0.0, 0.0),
                )
                .unwrap();
            }

            opencv::core::flip(&frame, &mut flipped_frame, 1).unwrap();

            if !ids.is_empty() {
                for i in 0..ids.len() {
                    let id = ids.get(i).unwrap();
                    let marker_corners = corners.get(i).unwrap();

                    let c0 = marker_corners.get(0).unwrap();
                    let c1 = marker_corners.get(1).unwrap();
                    let c2 = marker_corners.get(2).unwrap();
                    let c3 = marker_corners.get(3).unwrap();

                    let cx_raw = (c0.x + c1.x + c2.x + c3.x) / 4.0;
                    let cy_raw = (c0.y + c1.y + c2.y + c3.y) / 4.0;

                    let flipped_cx = 1920.0 - cx_raw;

                    let center_x = flipped_cx.max(0.0).min(1920.0) as u32;
                    let center_y = (1080.0 - cy_raw).max(0.0).min(1080.0) as u32;

                    let top_x = (c0.x + c1.x) / 2.0;
                    let top_y = (c0.y + c1.y) / 2.0;
                    let bot_x = (c2.x + c3.x) / 2.0;
                    let bot_y = (c2.y + c3.y) / 2.0;

                    let dx = top_x - bot_x;
                    let dy = bot_y - top_y;

                    let flipped_dx = -dx;
                    let heading = (flipped_dx.atan2(dy).to_degrees() + 360.0) % 360.0;

                    let _ = self
                        .pos_tx
                        .send([
                            /*id as u32,*/ center_x,
                            center_y,
                            heading.round() as u32,
                        ])
                        .await;

                    put_text(
                        &mut flipped_frame,
                        &format!("ID: {} | Cap: {} deg", id, heading.round()),
                        Point::new(flipped_cx as i32, (cy_raw - 15.0) as i32),
                        FONT_HERSHEY_SIMPLEX,
                        0.6,
                        Scalar::new(0.0, 255.0, 255.0, 0.0),
                        2,
                        LINE_8,
                        false,
                    )
                    .unwrap();
                }
            }

            highgui::imshow("Tracking ArUco", &flipped_frame).unwrap();
            if highgui::wait_key(1).unwrap() == 'q' as i32 {
                break;
            }
        }
    }

    async fn run_listener(&self) {
        self.logs_cli_tx()
            .send(LogEvent::System("Lancement de l'écoute GPS...".yellow()));

        let mut rx: MutexGuard<'_, Receiver<BitPacker>> = self.rx.lock().await;
        let mut pos_rx: MutexGuard<'_, Receiver<[u32; 3]>> = self.pos_rx.lock().await;

        loop {
            tokio::select! {
                Some(pos_arr) = pos_rx.recv() => {
                    self.set_coordinates(pos_arr[1], pos_arr[0], pos_arr[2]);
                },
                Some(msg) = rx.recv() => {
                    self.logs_cli_tx().send(LogEvent::Gps(format!("Demande de positionnement GPS reçue : {}", msg.to_bin_str()).green()));

                    let res = BitPacker::from_int(self.latitude.load(Ordering::Relaxed), Some(32)) + BitPacker::from_int(self.longitude.load(Ordering::Relaxed), Some(32)) + BitPacker::from_int(self.heading.load(Ordering::Relaxed), Some(32)) + msg;

                    self.antenna_tx.send(res.clone()).await;

                    self.logs_cli_tx().send(LogEvent::Gps(format!("Position GPS envoyée : {}", res.to_bin_str()).green()));
                }
            }
        }
    }

    pub fn start(self) -> (JoinHandle<()>, JoinHandle<()>) {
        self.logs_cli_tx()
            .send(LogEvent::System("Lancement du GPS...".yellow()));

        let listener_arc = Arc::new(self);
        let detect_and_send_arc = listener_arc.clone();
        let notification_arc = listener_arc.clone();

        let handles = (
            tokio::spawn(async move {
                detect_and_send_arc.run_detect_and_send().await;
            }),
            tokio::spawn(async move {
                listener_arc.run_listener().await;
            }),
        );

        notification_arc
            .logs_cli_tx()
            .send(LogEvent::System("GPS lancé.".yellow()));

        handles
    }
}
