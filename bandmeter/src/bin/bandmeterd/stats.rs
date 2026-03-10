use anyhow::Context;
use bandmeter_common::{Addr, Direction, Event};
use rusqlite::named_params;
use std::collections::HashMap;

use crate::util::*;

#[derive(Copy, Clone)]
struct TrafficStat {
    send: u64,
    recv: u64,
}

#[derive(Eq, PartialEq, Hash)]
struct TrafficStatKey {
    exe: Option<String>,
    raddr: String,
}

impl TrafficStatKey {
    fn new(pid: u32, raddr: &Addr) -> Self {
        Self {
            exe: get_exe(pid),
            raddr: parse_addr(&raddr),
        }
    }
}

pub struct Stats {
    stat_map: HashMap<TrafficStatKey, TrafficStat>,
    db: rusqlite::Connection,
}

impl Stats {
    pub fn new() -> anyhow::Result<Self> {
        let db = get_db()?;
        Ok(Self {
            stat_map: HashMap::with_capacity(256),
            db,
        })
    }

    pub fn update(&mut self, event: &Event) {
        self.stat_map
            .entry(TrafficStatKey::new(event.pid, &event.raddr))
            .and_modify(|stat| match event.direction {
                Direction::Recv => stat.recv += event.bytes as u64,
                Direction::Send => stat.send += event.bytes as u64,
            })
            .or_insert_with(|| match event.direction {
                Direction::Recv => TrafficStat {
                    send: 0,
                    recv: event.bytes as u64,
                },
                Direction::Send => TrafficStat {
                    send: event.bytes as u64,
                    recv: 0,
                },
            });
    }

    pub fn flush(&mut self, timestamp: i64) -> anyhow::Result<()> {
        let mut db_insert = self
            .db
            .prepare(
                "INSERT INTO stats(timestamp_utc, exe, raddr, send, recv)
                    VALUES(:timestamp, :exe, :raddr, :send, :recv)",
            )
            .context("error preparing INSERT query")?;

        for (key, stat) in self.stat_map.drain() {
            db_insert
                .execute(named_params! {
                    ":timestamp": timestamp, ":exe": key.exe, ":raddr": key.raddr,
                    ":send": stat.send as i64, ":recv": stat.recv as i64
                })
                .context("error inserting into database")?;
        }

        Ok(())
    }
}
