use chrono::{DateTime, Datelike, Local, Timelike};
use colored::{ColoredString, Colorize};
use shared::common::utils::{dt_to_slots_idx, get_current_dt};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::{Mutex, MutexGuard},
};

const SYSTEM_LOGS_FILE_LOCK: Mutex<()> = Mutex::new(());
const AIS_LOGS_FILE_LOCK: Mutex<()> = Mutex::new(());
const GPS_LOGS_FILE_LOCK: Mutex<()> = Mutex::new(());
const SATCOM_LOGS_FILE_LOCK: Mutex<()> = Mutex::new(());
const COMPUTER_LOGS_FILE_LOCK: Mutex<()> = Mutex::new(());

pub fn log(msg: ColoredString, log_filename: &str, log_file_lock: Mutex<()>) -> () {
    let current_dt: DateTime<Local> = get_current_dt();
    let slots: [u16; 2] = dt_to_slots_idx(Some(current_dt));
    let log_msg: String = format!(
        "({}, {}), {}/{}/{} {}h:{}mn:{}s :\n\t{}\n",
        slots[0],
        slots[1],
        current_dt.day(),
        current_dt.month(),
        current_dt.year(),
        current_dt.hour(),
        current_dt.minute(),
        current_dt.second(),
        msg.clone()
    );
    let file_log_msg: String = format!(
        "\n        {}\n({}, {}), {}/{}/{} {}h:{}mn:{}s:",
        msg.clear(),
        slots[0],
        slots[1],
        current_dt.day(),
        current_dt.month(),
        current_dt.year(),
        current_dt.hour(),
        current_dt.minute(),
        current_dt.second()
    );

    let _unused: MutexGuard<'_, ()> = log_file_lock.lock().unwrap();

    let mut log_file: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{}.log", log_filename))
        .unwrap();

    let _ = writeln!(log_file, "{file_log_msg}");

    println!("{log_msg}");
}

pub fn system_log(msg: ColoredString) -> () {
    log(msg, "system_logs", SYSTEM_LOGS_FILE_LOCK);
}

pub fn ais_log(msg: ColoredString) -> () {
    log(msg, "ais_logs", AIS_LOGS_FILE_LOCK);
}

pub fn gps_log(msg: ColoredString) -> () {
    log(msg, "gps_logs", GPS_LOGS_FILE_LOCK);
}

pub fn satcom_log(msg: ColoredString) -> () {
    log(msg, "satcom_logs", SATCOM_LOGS_FILE_LOCK);
}

pub fn computer_log(msg: ColoredString) -> () {
    log(msg, "computer_logs", COMPUTER_LOGS_FILE_LOCK);
}
