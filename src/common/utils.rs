use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::Mutex,
};

use chrono::{DateTime, Datelike, Local, Timelike};
use colored::{ColoredString, Colorize};
use shared::common::utils::{datetime_to_slots_idx, get_current_datetime};

static LOG_FILE_LOCK: Mutex<()> = Mutex::new(());

pub fn log(msg: ColoredString) -> () {
    let cdt: DateTime<Local> = get_current_datetime();
    let slots: [u16; 2] = datetime_to_slots_idx(Some(cdt));
    let log_msg: String = format!(
        "({}, {}), {}/{}/{} {}h:{}mn:{}s :\n\t{}\n",
        slots[0],
        slots[1],
        cdt.day(),
        cdt.month(),
        cdt.year(),
        cdt.hour(),
        cdt.minute(),
        cdt.second(),
        msg.clone()
    );
    let file_log_msg: String = format!(
        "\n        {}\n({}, {}), {}/{}/{} {}h:{}mn:{}s:",
        msg.clear(),
        slots[0],
        slots[1],
        cdt.day(),
        cdt.month(),
        cdt.year(),
        cdt.hour(),
        cdt.minute(),
        cdt.second()
    );

    let lock: std::sync::MutexGuard<'_, ()> = LOG_FILE_LOCK.lock().unwrap();

    let mut log_file: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs.log")
        .unwrap();

    let _ = writeln!(log_file, "{file_log_msg}");

    println!("{log_msg}");
}
