use chrono::{DateTime, Datelike, Local, Timelike};
use colored::{ColoredString, Colorize};
use shared::common::utils::{dt_to_slots_idx, get_current_dt};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::Mutex,
};

static LOG_FILE_LOCK: Mutex<()> = Mutex::new(());

pub fn log(msg: ColoredString) -> () {
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

    let _unused: std::sync::MutexGuard<'_, ()> = LOG_FILE_LOCK.lock().unwrap();

    let mut log_file: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs.log")
        .unwrap();

    writeln!(log_file, "{file_log_msg}");

    println!("{log_msg}");
}
