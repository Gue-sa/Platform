use std::sync::{Arc, atomic::AtomicU32};

use shared::bitpacker::BitPacker;
use tokio::sync::{
    Mutex, Semaphore,
    mpsc::{Receiver, Sender, channel},
};

use opencv::{
    core::{self, Scalar},
    highgui,
    imgproc::{self, INTER_CUBIC},
    prelude::*,
    videoio::{
        self, CAP_ANY, CAP_PROP_BUFFERSIZE, CAP_PROP_EXPOSURE, CAP_PROP_FRAME_HEIGHT,
        CAP_PROP_FRAME_WIDTH, CAP_V4L, CAP_V4L2, VideoCapture,
    },
};

pub struct HarbourmasterGps {
    rx: Mutex<Receiver<BitPacker>>,
    pos_rx: Mutex<Receiver<[u32; 2]>>,
    pos_tx: Sender<[u32; 2]>,
    antenna_tx: Sender<BitPacker>,
    latitude: AtomicU32,
    longitude: AtomicU32,
}

impl HarbourmasterGps {
    pub async fn init(rx: Receiver<BitPacker>, ant_tx: Sender<BitPacker>) -> Self {
        let (pos_tx, pos_rx) = channel::<[u32; 2]>(Semaphore::MAX_PERMITS);

        Self {
            rx: Mutex::new(rx),
            pos_rx: Mutex::new(pos_rx),
            pos_tx: pos_tx,
            antenna_tx: ant_tx,
            latitude: AtomicU32::new(0),
            longitude: AtomicU32::new(0),
        }
    }

    fn coordinates(&self) -> [u32; 2] {
        [
            self.latitude.load(std::sync::atomic::Ordering::Relaxed),
            self.longitude.load(std::sync::atomic::Ordering::Relaxed),
        ]
    }

    fn set_coordinates(&self, lat: u32, lon: u32) {
        self.latitude
            .store(lat, std::sync::atomic::Ordering::Relaxed);
        self.longitude
            .store(lon, std::sync::atomic::Ordering::Relaxed);
    }

    async fn run_detect_and_send(&self) -> () {
        let mut cam = VideoCapture::new(4, CAP_V4L2).unwrap();

        let fourcc = videoio::VideoWriter::fourcc('M', 'J', 'P', 'G').unwrap();
        cam.set(videoio::CAP_PROP_FOURCC, fourcc as f64).unwrap();

        cam.set(videoio::CAP_PROP_FRAME_WIDTH, 1920.).unwrap();
        cam.set(videoio::CAP_PROP_FRAME_HEIGHT, 1080.).unwrap();

        cam.set(CAP_PROP_BUFFERSIZE, 1.);

        //cam.set(videoio::CAP_PROP_AUTO_EXPOSURE, 1.);
        //cam.set(CAP_PROP_EXPOSURE, -3.).unwrap();

        let mut frame: Mat = Mat::default();
        let mut flipped_frame: Mat = Mat::default();
        let mut processed_mask: Mat = Mat::default();
        let mut hsv: Mat = Mat::default();

        //let mut blurred = Mat::default();

        let mut mask1: Mat = Mat::default();
        let mut mask2: Mat = Mat::default();

        let mut mask: Mat = Mat::default();

        let mut contours: core::Vector<core::Vector<core::Point_<i32>>> =
            core::Vector::<core::Vector<core::Point>>::new();

        let kernel: Mat = imgproc::get_structuring_element(
            imgproc::MORPH_RECT,
            core::Size::new(5, 5),
            core::Point::new(-1, -1),
        )
        .unwrap();

        let mut approx: core::Vector<core::Point_<i32>> = core::Vector::<core::Point>::new();

        loop {
            cam.read(&mut frame).unwrap();
            if frame.empty() {
                break;
            }

            core::flip(&frame, &mut flipped_frame, 1).unwrap();

            imgproc::cvt_color(
                &flipped_frame,
                &mut hsv,
                imgproc::COLOR_BGR2HSV,
                0,
                core::AlgorithmHint::ALGO_HINT_DEFAULT,
            )
            .unwrap();

            // --- SEUILLAGE DU ROUGE ---
            // On baisse la saturation minimale (S=70 -> 50) pour capter le rouge "lavé" par la lumière de l'écran
            core::in_range(
                &hsv,
                &Scalar::new(0.0, 160.0, 100.0, 0.0),
                &Scalar::new(10.0, 255.0, 255.0, 0.0),
                &mut mask1,
            )
            .unwrap();
            core::in_range(
                &hsv,
                &Scalar::new(170.0, 160.0, 100.0, 0.0),
                &Scalar::new(180.0, 255.0, 255.0, 0.0),
                &mut mask2,
            )
            .unwrap();

            core::add(&mask1, &mask2, &mut mask, &core::no_array(), -1).unwrap();
            //core::add(&mask1, &mask2, &mut combined_mask, &core::no_array(), -1).unwrap();

            // Nettoyage morphologique pour boucher les trous dus aux reflets
            imgproc::morphology_ex(
                &mask,
                &mut processed_mask,
                imgproc::MORPH_CLOSE,
                &kernel,
                core::Point::new(-1, -1),
                1,
                core::BORDER_CONSTANT,
                Scalar::default(),
            )
            .unwrap();

            imgproc::find_contours(
                &processed_mask,
                &mut contours,
                imgproc::RETR_EXTERNAL,
                imgproc::CHAIN_APPROX_SIMPLE,
                core::Point::new(0, 0),
            )
            .unwrap();

            for contour in contours.iter() {
                let area: f64 = imgproc::contour_area(&contour, false).unwrap();
                if area > 800.0 {
                    let perimeter = imgproc::arc_length(&contour, true).unwrap();

                    // On augmente légèrement la précision (0.02 au lieu de 0.04)
                    imgproc::approx_poly_dp(&contour, &mut approx, 0.02 * perimeter, true).unwrap();

                    // Détection de RECTANGLE (4 sommets + convexe)
                    if approx.len() == 4 && imgproc::is_contour_convex(&approx).unwrap() {
                        let rect: core::Rect_<i32> = imgproc::bounding_rect(&approx).unwrap();

                        let moments: core::Moments = imgproc::moments(&contour, false).unwrap();
                        let mut center_x: u32 = 0;
                        let mut center_y: u32 = 0;

                        if moments.m00 != 0.0 {
                            // Calcul du centre de masse (barycentre)
                            center_x = ((moments.m10 / moments.m00) as u32).min(1920).max(0);
                            center_y = ((1080. - (moments.m01 / moments.m00)) as u32)
                                .min(1080)
                                .max(0);

                            // Si tu as fait un downscaling, multiplie par 2 pour dessiner sur l'image originale
                            // let final_x = center_x * 2;
                        }

                        self.pos_tx.send([center_x, center_y]).await;

                        // Dessin des résultats
                        imgproc::rectangle(
                            &mut flipped_frame,
                            rect,
                            Scalar::new(0.0, 255.0, 0.0, 0.0),
                            2,
                            imgproc::LINE_8,
                            0,
                        )
                        .unwrap();
                        imgproc::put_text(
                            &mut flipped_frame,
                            &format!("ID: {}x{}", rect.width, rect.height),
                            core::Point::new(rect.x, rect.y - 5),
                            imgproc::FONT_HERSHEY_SIMPLEX,
                            0.5,
                            Scalar::new(0.0, 255.0, 0.0, 0.0),
                            1,
                            imgproc::LINE_8,
                            false,
                        )
                        .unwrap();
                    }
                }
            }

            highgui::imshow("Tracking Multi-Rectangles", &flipped_frame).unwrap();
            if highgui::wait_key(1).unwrap() == 'q' as i32 {
                break;
            }
        }
    }

    async fn run_listener(&self) -> () {
        let mut rx = self.rx.lock().await;
        let mut pos_rx = self.pos_rx.lock().await;

        loop {
            tokio::select! {
                Some(pos_arr) = pos_rx.recv() => {
                    self.set_coordinates(pos_arr[1], pos_arr[0]);
                },
                Some(msg) = rx.recv() => {
                    println!("Requête GPS reçue : {:?}", msg);

                    let res: BitPacker = BitPacker::from_int(self.latitude.load(std::sync::atomic::Ordering::Relaxed), Some(32)) + BitPacker::from_int(self.longitude.load(std::sync::atomic::Ordering::Relaxed), Some(32)) + msg;

                    self.antenna_tx.send(res).await;
                }
            }
        }
    }

    pub async fn start(self) -> () {
        let listener_arc = Arc::new(self);
        let detect_and_send_arc = listener_arc.clone();

        tokio::spawn(async move {
            detect_and_send_arc.run_detect_and_send().await;
        });

        tokio::spawn(async move {
            listener_arc.run_listener().await;
        });
    }
}
