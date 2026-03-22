use anyhow::{Context, anyhow};
use gpui::SharedString;
use rusqlite::Connection;
use std::env;

use crate::{Stat, period::Period};

pub const DB_NAME: &str = "bandmeter.db";

pub fn db_dir() -> anyhow::Result<String> {
    env::var("DB_DIR").context("environment variable \"DB_DIR\" not provided")
}

pub struct DBManager {
    conn: Option<Connection>,
}

impl DBManager {
    pub fn new() -> Self {
        use rusqlite::OpenFlags;

        // See beginning of `main`
        let db_dir = db_dir().unwrap();
        let conn = Connection::open_with_flags(
            format!("{db_dir}/{DB_NAME}"),
            OpenFlags::SQLITE_OPEN_READ_ONLY
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| eprintln!("Error: {:?}", anyhow::format_err!(e)))
        .ok();

        Self { conn }
    }

    fn do_query<T, P, F>(
        &mut self,
        sql: &str,
        params: P,
        f: F,
        capacity: usize,
    ) -> anyhow::Result<Vec<T>>
    where
        P: rusqlite::Params,
        F: FnMut(&rusqlite::Row) -> rusqlite::Result<T>,
    {
        let mut result = Vec::with_capacity(capacity);

        let conn = self
            .conn
            .as_ref()
            .ok_or_else(|| anyhow!("could not establish connection to database"))?;
        let mut stmt = conn.prepare(sql)?;

        let rows_iter = stmt.query_map(params, f)?;

        for row in rows_iter {
            result.push(row?);
        }

        Ok(result)
    }

    pub fn query_raw(&mut self, period: &Period) -> Vec<Stat> {
        let (start, end) = period.bounds().timestamp();
        self.do_query(
            "SELECT timestamp_utc, exe, raddr, send, recv FROM stats WHERE timestamp_utc >= ?1 AND timestamp_utc < ?2",
            [start, end],
            |row| {
                let exe: Option<SharedString> = row.get::<_, Option<String>>(1)?.map(String::into);

                Ok(Stat {
                    timestamp: row.get(0)?,
                    exe,
                    raddr: row.get::<_, String>(2)?.into(),
                    send: row.get::<_, i64>(3)? as u64,
                    recv: row.get::<_, i64>(4)? as u64,
                })
            },
            period.segments().count() // Maybe implement size_hint()?
        ).context("error querying database").map_err(|e| eprintln!("Error: {e:?}"))
        .ok()
        .unwrap_or_else(Vec::new)
    }

    // ** Did a simple timing test and this seemed slower than `filter_stats`
    //
    // pub fn query_apps(&mut self, period: Period) -> Vec<AppStat> {
    //     let (start, end) = period.bounds();
    //     self.do_query(
    //         "SELECT exe, sum(send), sum(recv) FROM stats where timestamp_utc >= ?1 AND timestamp_utc < ?2 GROUP BY exe ORDER BY sum(send) + sum(recv) DESC",
    //         [start, end],
    //         |row| {
    //             let exe: Option<SharedString> = row.get::<_, Option<String>>(1)?.map(String::into);

    //             Ok(AppStat {
    //                 exe,
    //                 download: row.get::<_, i64>(1)? as u64,
    //                 upload: row.get::<_, i64>(2)? as u64,
    //             })
    //         },
    //         period.num_divisions()
    //     )
    //     .ok()
    //     .unwrap_or_else(Vec::new)
    // }
}
