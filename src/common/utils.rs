use std::{fs::{File, OpenOptions}, io::Write, sync::Mutex};

use chrono::{DateTime, Datelike, Local, Timelike};
use colored::{ColoredString, Colorize};

use crate::shared::common::constants::SLOTS_PER_MINUTE;


static LOG_FILE_LOCK: Mutex<()> = Mutex::new(());


pub fn get_current_datetime() -> DateTime<Local> {
    Local::now()
}


pub fn get_timestamp(datetime: Option<DateTime<Local>>) -> i64 {
    let datetime: DateTime<Local> = datetime.unwrap_or(Local::now());
    datetime.timestamp()
}


pub fn datetime_to_slots_idx(datetime: Option<DateTime<Local>>) -> [u16; 2] {
    let datetime: DateTime<Local> = datetime.unwrap_or(Local::now());
    let si: u16 = ((datetime.second() * 1000 + datetime.timestamp_subsec_millis()) * SLOTS_PER_MINUTE as u32 / 60_000) as u16;
    [si, si + SLOTS_PER_MINUTE]
}


pub fn log(msg: ColoredString) -> () {
    let cdt: DateTime<Local> = get_current_datetime();
    let slots: [u16; 2] = datetime_to_slots_idx(Some(cdt));
    let log_msg: String = format!("({}, {}), {}/{}/{} {}h:{}mn:{}s :\n\t{}\n", slots[0], slots[1], cdt.day(), cdt.month(), cdt.year(), cdt.hour(), cdt.minute(), cdt.second(), msg.clone());
    let file_log_msg: String = format!("\n        {}\n({}, {}), {}/{}/{} {}h:{}mn:{}s:", msg.clear(), slots[0], slots[1], cdt.day(), cdt.month(), cdt.year(), cdt.hour(), cdt.minute(), cdt.second());

    let lock: std::sync::MutexGuard<'_, ()> = LOG_FILE_LOCK.lock().unwrap();

    let mut log_file: File = OpenOptions::new().create(true).append(true).open("logs.log").unwrap();

    let _ = writeln!(log_file, "{file_log_msg}");

    println!("{log_msg}");
}