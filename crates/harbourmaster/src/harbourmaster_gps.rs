use colored::Colorize;
use opencv::{
    core::{
        AlgorithmHint::ALGO_HINT_DEFAULT, BORDER_CONSTANT, Point, Point_, Rect_, Scalar, Size,
        Vector, add, flip, in_range, no_array,
    },
    imgproc::{
        CHAIN_APPROX_SIMPLE, COLOR_BGR2HSV, FONT_HERSHEY_SIMPLEX, LINE_8, MORPH_CLOSE, MORPH_RECT,
        RETR_EXTERNAL, approx_poly_dp, arc_length, bounding_rect, contour_area, cvt_color,
        find_contours, get_structuring_element, is_contour_convex, moments, morphology_ex,
        put_text, rectangle,
    },
    prelude::*,
    videoio::{
        CAP_PROP_BUFFERSIZE, CAP_PROP_FOURCC, CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH,
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
    pos_rx: Mutex<Receiver<[u32; 2]>>,
    pos_tx: Sender<[u32; 2]>,
    antenna_tx: Sender<BitPacker>,
    latitude: AtomicU32,
    longitude: AtomicU32,
    logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
}

impl HarbourmasterGps {
    pub async fn init(
        rx: Receiver<BitPacker>,
        ant_tx: Sender<BitPacker>,
        cli_tx: std::sync::mpsc::Sender<LogEvent>,
    ) -> Self {
        let (pos_tx, pos_rx) = channel::<[u32; 2]>(Semaphore::MAX_PERMITS);

        Self {
            rx: Mutex::new(rx),
            pos_rx: Mutex::new(pos_rx),
            pos_tx: pos_tx,
            antenna_tx: ant_tx,
            latitude: AtomicU32::new(0),
            longitude: AtomicU32::new(0),
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

    fn set_coordinates(&self, lat: u32, lon: u32) {
        self.latitude.store(lat, Ordering::Relaxed);
        self.longitude.store(lon, Ordering::Relaxed);
    }

    async fn run_detect_and_send(&self) -> () {
        self.logs_cli_tx()
            .send(LogEvent::System("Lancement du satellite GPS...".yellow()));

        let mut cam = VideoCapture::new(
            Config::load().unwrap().gps_cam_idx().unwrap().into(),
            CAP_V4L2,
        )
        .unwrap();

        let fourcc = VideoWriter::fourcc('M', 'J', 'P', 'G').unwrap();
        cam.set(CAP_PROP_FOURCC, fourcc.into()).unwrap();

        cam.set(CAP_PROP_FRAME_WIDTH, 1920.).unwrap();
        cam.set(CAP_PROP_FRAME_HEIGHT, 1080.).unwrap();

        cam.set(CAP_PROP_BUFFERSIZE, 1.).unwrap();

        //cam.set(videoio::CAP_PROP_AUTO_EXPOSURE, 1.);
        //cam.set(CAP_PROP_EXPOSURE, -3.).unwrap();

        let mut frame = Mat::default();
        let mut flipped_frame = Mat::default();
        let mut processed_mask = Mat::default();
        let mut hsv = Mat::default();

        let mut mask1 = Mat::default();
        let mut mask2 = Mat::default();

        let mut mask = Mat::default();

        let mut contours: Vector<Vector<Point_<i32>>> = Vector::<Vector<Point>>::new();

        let kernel: Mat =
            get_structuring_element(MORPH_RECT, Size::new(5, 5), Point::new(-1, -1)).unwrap();

        let mut approx = Vector::<Point>::new();

        loop {
            cam.read(&mut frame).unwrap();
            if frame.empty() {
                break;
            }

            flip(&frame, &mut flipped_frame, 1).unwrap();

            #[cfg(any(
                feature = "arch-based",
                feature = "rasp-based",
                not(any(
                    feature = "arch-based",
                    feature = "rasp-based",
                    feature = "debian-based"
                ))
            ))]
            cvt_color(
                &flipped_frame,
                &mut hsv,
                COLOR_BGR2HSV,
                0,
                ALGO_HINT_DEFAULT,
            )
            .unwrap();

            #[cfg(feature = "debian-based")]
            cvt_color(&flipped_frame, &mut hsv, COLOR_BGR2HSV, 0).unwrap();

            in_range(
                &hsv,
                &Scalar::new(0.0, 160.0, 100.0, 0.0),
                &Scalar::new(10.0, 255.0, 255.0, 0.0),
                &mut mask1,
            )
            .unwrap();

            in_range(
                &hsv,
                &Scalar::new(170.0, 160.0, 100.0, 0.0),
                &Scalar::new(180.0, 255.0, 255.0, 0.0),
                &mut mask2,
            )
            .unwrap();

            add(&mask1, &mask2, &mut mask, &no_array(), -1).unwrap();

            morphology_ex(
                &mask,
                &mut processed_mask,
                MORPH_CLOSE,
                &kernel,
                Point::new(-1, -1),
                1,
                BORDER_CONSTANT,
                Scalar::default(),
            )
            .unwrap();

            find_contours(
                &processed_mask,
                &mut contours,
                RETR_EXTERNAL,
                CHAIN_APPROX_SIMPLE,
                Point::new(0, 0),
            )
            .unwrap();

            for contour in contours.iter() {
                let area = contour_area(&contour, false).unwrap();
                if area > 800.0 {
                    let perimeter = arc_length(&contour, true).unwrap();

                    approx_poly_dp(&contour, &mut approx, 0.02 * perimeter, true).unwrap();

                    if approx.len() == 4 && is_contour_convex(&approx).unwrap() {
                        let rect: Rect_<i32> = bounding_rect(&approx).unwrap();

                        let moments = moments(&contour, false).unwrap();
                        let mut center_x = 0;
                        let mut center_y = 0;

                        if moments.m00 != 0.0 {
                            // Calcul du centre de masse (barycentre)
                            center_x = ((moments.m10 / moments.m00) as u32).min(1920).max(0);
                            center_y = ((1080. - (moments.m01 / moments.m00)) as u32)
                                .min(1080)
                                .max(0);

                            // Si downscaling, multiplier par 2 pour dessiner sur l'image originale
                            // let final_x = center_x * 2;
                        }

                        self.pos_tx.send([center_x, center_y]).await;

                        rectangle(
                            &mut flipped_frame,
                            rect,
                            Scalar::new(0.0, 255.0, 0.0, 0.0),
                            2,
                            LINE_8,
                            0,
                        )
                        .unwrap();
                        put_text(
                            &mut flipped_frame,
                            &format!("ID: {}x{}", rect.width, rect.height),
                            Point::new(rect.x, rect.y - 5),
                            FONT_HERSHEY_SIMPLEX,
                            0.5,
                            Scalar::new(0.0, 255.0, 0.0, 0.0),
                            1,
                            LINE_8,
                            false,
                        )
                        .unwrap();
                    }
                }
            }

            /*
            highgui::imshow("Tracking Multi-Rectangles", &flipped_frame).unwrap();
            if highgui::wait_key(1).unwrap() == 'q' as i32 {
                break;
            }
            */
        }
    }

    async fn run_listener(&self) -> () {
        self.logs_cli_tx()
            .send(LogEvent::System("Lancement de l'écoute GPS...".yellow()));

        let mut rx: MutexGuard<'_, Receiver<BitPacker>> = self.rx.lock().await;
        let mut pos_rx: MutexGuard<'_, Receiver<[u32; 2]>> = self.pos_rx.lock().await;

        loop {
            tokio::select! {
                Some(pos_arr) = pos_rx.recv() => {
                    self.set_coordinates(pos_arr[1], pos_arr[0]);
                },
                Some(msg) = rx.recv() => {
                    self.logs_cli_tx().send(LogEvent::Gps(format!("Demande de positionnement GPS reçue : {}", msg.to_bin_str()).green()));

                    let res = BitPacker::from_int(self.latitude.load(Ordering::Relaxed), Some(32)) + BitPacker::from_int(self.longitude.load(Ordering::Relaxed), Some(32)) + msg;

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

        (
            tokio::spawn(async move {
                detect_and_send_arc.run_detect_and_send().await;
            }),
            tokio::spawn(async move {
                listener_arc.run_listener().await;
            }),
        )
    }
}
