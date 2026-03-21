use chrono::{
    DateTime, Days, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Timelike,
};
use gpui::SharedString;
use gpui_component::select::SelectItem;

pub const SECS_MIN: i64 = 60;
pub const SECS_HOUR: i64 = SECS_MIN * 60;

#[derive(Debug, Clone)]
pub enum PeriodType {
    Hour,
    Day,
}

impl SelectItem for PeriodType {
    type Value = Self;

    fn title(&self) -> SharedString {
        format!("{:?}", self).into()
    }

    fn value(&self) -> &Self::Value {
        &self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Period {
    Hour(DateTime<Local>),
    Day(NaiveDate),
}

impl Period {
    pub fn default() -> Self {
        Self::current(&PeriodType::Day)
    }

    pub fn current(period_type: &PeriodType) -> Self {
        let now = Local::now();

        match period_type {
            PeriodType::Hour => {
                let hour = now.with_hour_only(now.hour());

                Period::Hour(hour)
            }
            PeriodType::Day => {
                let day = now.date_naive();

                Period::Day(day)
            }
        }
    }

    pub fn is_current(&self) -> bool {
        Local::now() < self.bounds().1
    }

    pub fn bounds(&self) -> (DateTime<Local>, DateTime<Local>) {
        match *self {
            Period::Hour(h) => (h, h.next_hour()),
            Period::Day(d) => {
                let start = d.at_midnight().to_local();
                let end = d.next().at_midnight().to_local();

                (start, end)
            }
        }
    }

    pub fn bounds_timestamp(&self) -> (i64, i64) {
        let (start, end) = self.bounds();
        (start.timestamp(), end.timestamp())
    }

    pub fn intvl_secs(&self) -> i64 {
        match self {
            Period::Hour(_) => SECS_MIN * 2, // 2 min
            Period::Day(_) => SECS_HOUR,
        }
    }

    pub fn num_divisions(&self) -> usize {
        let (period_start, period_end) = self.bounds_timestamp();
        let intvl = self.intvl_secs();

        (period_start..period_end).step_by(intvl as usize).count()
    }

    pub fn prev(&mut self) -> bool {
        match *self {
            Period::Hour(h) => *self = Period::Hour(h.prev_hour()),
            Period::Day(d) => *self = Period::Day(d.prev()),
        }

        true
    }

    pub fn next(&mut self) -> bool {
        if self.is_current() {
            return false;
        }

        match *self {
            Period::Hour(h) => *self = Period::Hour(h.next_hour()),
            Period::Day(d) => *self = Period::Day(d.next()),
        }

        true
    }

    pub fn switch(&mut self, to_type: &PeriodType) {
        if self.is_current() {
            *self = Period::current(to_type);
            return;
        }

        let start = self.bounds().0;

        match to_type {
            PeriodType::Hour => *self = Period::Hour(start.with_hour_only(start.hour())),
            PeriodType::Day => *self = Period::Day(start.date_naive()),
        }
    }
}

// Extensions for convenience

pub trait NaiveDateExt {
    fn at_midnight(&self) -> NaiveDateTime;
    fn prev(self) -> NaiveDate;
    fn next(self) -> NaiveDate;
}

impl NaiveDateExt for NaiveDate {
    fn at_midnight(&self) -> NaiveDateTime {
        self.and_hms_opt(0, 0, 0).unwrap()
    }

    fn prev(self) -> NaiveDate {
        self.checked_sub_days(Days::new(1)).unwrap()
    }

    fn next(self) -> NaiveDate {
        self.checked_add_days(Days::new(1)).unwrap()
    }
}

pub trait NaiveDateTimeExt {
    fn to_local(&self) -> DateTime<Local>;
}

impl NaiveDateTimeExt for NaiveDateTime {
    fn to_local(&self) -> DateTime<Local> {
        Local.from_local_datetime(self).unwrap()
    }
}

pub trait DateTimeLocalExt {
    fn with_hour_only(&self, hour: u32) -> DateTime<Local>;
    fn prev_hour(self) -> DateTime<Local>;
    fn next_hour(self) -> DateTime<Local>;
}

impl DateTimeLocalExt for DateTime<Local> {
    fn prev_hour(self) -> DateTime<Local> {
        self.checked_sub_signed(TimeDelta::hours(1)).unwrap()
    }

    fn next_hour(self) -> DateTime<Local> {
        self.checked_add_signed(TimeDelta::hours(1)).unwrap()
    }

    fn with_hour_only(&self, hour: u32) -> DateTime<Local> {
        self.with_time(NaiveTime::from_hms_opt(hour, 0, 0).unwrap())
            .unwrap()
    }
}
