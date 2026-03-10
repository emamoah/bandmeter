use anyhow::Context;
use bandmeter_common::Addr;
use chrono::Utc;
use std::{
    env,
    fs::read_link,
    net::{Ipv4Addr, Ipv6Addr},
};

pub static DB_NAME: &str = "bandmeter.db";

pub fn get_timestamp() -> i64 {
    Utc::now().timestamp()
}

pub fn get_exe(pid: u32) -> Option<String> {
    // Further optimisation possible?
    read_link(format!("/proc/{}/exe", pid))
        .ok()
        .map(|path_buf| {
            path_buf
                .to_string_lossy()
                .trim_end_matches(" (deleted)")
                .into()
        })
}

pub fn parse_addr(addr: &Addr) -> String {
    match addr {
        Addr::Addr4(addr) => Ipv4Addr::from_bits(u32::from_be(*addr)).to_string(),
        Addr::Addr6(addr) => Ipv6Addr::from_octets(*addr).to_string(),
    }
}

pub fn db_dir() -> anyhow::Result<String> {
    env::var("DB_DIR").context("environment variable \"DB_DIR\" not provided")
}

pub fn get_db() -> anyhow::Result<rusqlite::Connection> {
    let db_dir = db_dir()?;
    rusqlite::Connection::open(format!("{db_dir}/{DB_NAME}")).context("error opening database")
}

// Logging (formatted for systemd)

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        eprintln!("<3>{}", format!($($arg)+).replace("\n", "\n<3>"))
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        eprintln!("<4>{}", format!($($arg)+).replace("\n", "\n<4>"))
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        eprintln!("<6>{}", format!($($arg)+).replace("\n", "\n<6>"))
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        eprintln!("<7>{}", format!($($arg)+).replace("\n", "\n<7>"))
    }
}
