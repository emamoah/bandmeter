use chrono::Utc;

pub const SECS_MIN: i64 = 60;
pub const SECS_HOUR: i64 = SECS_MIN * 60;
pub const SECS_DAY: i64 = SECS_HOUR * 24;

pub const PERIOD_HOUR: &str = "Hour";
pub const PERIOD_DAY: &str = "Day";

#[derive(Debug, Clone, Copy)]
pub enum Period {
    Hour(i64),
    Day(i64),
}

impl Period {
    pub fn current(period_type: &str) -> Self {
        let now = Utc::now().timestamp();
        match period_type {
            PERIOD_HOUR => Period::Hour(now - now % SECS_HOUR),
            PERIOD_DAY | _ => Period::Day(now - now % SECS_DAY),
        }
    }

    pub fn bounds(&self) -> (i64, i64) {
        match *self {
            Period::Hour(h) => (h, h + SECS_HOUR),
            Period::Day(d) => (d, d + SECS_DAY),
        }
    }

    pub fn intvl_secs(&self) -> i64 {
        match self {
            Period::Hour(_) => SECS_MIN * 2, // 2 min
            Period::Day(_) => SECS_HOUR,
        }
    }

    pub fn num_divisions(&self) -> usize {
        let (period_start, period_end) = self.bounds();
        let intvl = self.intvl_secs();

        (period_start..period_end).step_by(intvl as usize).count()
    }

    pub fn prev(&mut self) {
        match *self {
            // Unchecked arithmetic because it's *impractical* to under/overflow
            Period::Hour(h) => *self = Period::Hour(h - SECS_HOUR),
            Period::Day(d) => *self = Period::Day(d - SECS_DAY),
        }
    }

    pub fn next(&mut self) {
        match *self {
            Period::Hour(h) => *self = Period::Hour(h + SECS_HOUR),
            Period::Day(d) => *self = Period::Day(d + SECS_DAY),
        }
    }
}
