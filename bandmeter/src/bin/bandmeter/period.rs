use std::cmp::min;

use chrono::{
    DateTime, Datelike, Days, Local, Months, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta,
    TimeZone, Timelike, Weekday,
};
use gpui::SharedString;
use gpui_component::select::SelectItem;

#[derive(Debug, Clone, Copy)]
pub enum PeriodType {
    Hour,
    Day,
    Week,
    Month,
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

pub struct TimeBounds(pub DateTime<Local>, pub DateTime<Local>);

impl TimeBounds {
    pub fn timestamp(&self) -> (i64, i64) {
        (self.0.timestamp(), self.1.timestamp())
    }
}

pub struct Segments {
    period_type: PeriodType,
    seg_start: DateTime<Local>,
    period_end: DateTime<Local>,
}

impl Segments {
    pub fn timestamp(&mut self) -> impl Iterator<Item = (i64, i64)> {
        self.into_iter().map(|s| s.timestamp())
    }
}

impl Iterator for Segments {
    type Item = TimeBounds;

    fn next(&mut self) -> Option<Self::Item> {
        if self.seg_start == self.period_end {
            return None;
        }

        let seg_end = min(
            self.period_end,
            match self.period_type {
                PeriodType::Hour => self
                    .seg_start
                    .checked_add_signed(TimeDelta::minutes(2))
                    .unwrap(),
                PeriodType::Day => self.seg_start.next_hour(),
                PeriodType::Week | PeriodType::Month => {
                    self.seg_start.date_naive().next().at_midnight().to_local()
                }
            },
        );

        let curr = TimeBounds(self.seg_start, seg_end);

        self.seg_start = seg_end;

        Some(curr)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Period {
    Hour(DateTime<Local>),
    Day(NaiveDate),
    Week(NaiveDate),
    Month(NaiveDate),
}

impl Period {
    pub fn default() -> Self {
        Self::current(&PeriodType::Day)
    }

    pub fn current(period_type: &PeriodType) -> Self {
        let now = Local::now();

        match period_type {
            PeriodType::Hour => Period::Hour(now.with_hour_only(now.hour())),
            PeriodType::Day => Period::Day(now.date_naive()),
            PeriodType::Week => Period::Week(now.date_naive().week(Weekday::Sun).first_day()),
            PeriodType::Month => Period::Month(now.date_naive().with_day(1).unwrap()),
        }
    }

    pub fn period_type(&self) -> PeriodType {
        match self {
            Period::Hour(_) => PeriodType::Hour,
            Period::Day(_) => PeriodType::Day,
            Period::Week(_) => PeriodType::Week,
            Period::Month(_) => PeriodType::Month,
        }
    }

    pub fn segments(&self) -> Segments {
        let TimeBounds(start, end) = self.bounds();

        Segments {
            period_type: self.period_type(),
            seg_start: start,
            period_end: end,
        }
    }

    pub fn is_current(&self) -> bool {
        Local::now() < self.bounds().1
    }

    pub fn bounds(&self) -> TimeBounds {
        match *self {
            Period::Hour(h) => TimeBounds(h, h.next_hour()),
            Period::Day(d) => {
                let start = d.at_midnight().to_local();
                let end = d.next().at_midnight().to_local();

                TimeBounds(start, end)
            }
            Period::Week(w) => {
                let start = w.at_midnight().to_local();
                let end = w.add_days(7).at_midnight().to_local();

                TimeBounds(start, end)
            }
            Period::Month(m) => {
                let start = m.at_midnight().to_local();
                let end = m.add_months(1).at_midnight().to_local();

                TimeBounds(start, end)
            }
        }
    }

    pub fn prev(&mut self) -> bool {
        match *self {
            Period::Hour(h) => *self = Period::Hour(h.prev_hour()),
            Period::Day(d) => *self = Period::Day(d.prev()),
            Period::Week(w) => *self = Period::Week(w.sub_days(7)),
            Period::Month(m) => *self = Period::Month(m.sub_months(1)),
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
            Period::Week(w) => *self = Period::Week(w.add_days(7)),
            Period::Month(m) => *self = Period::Month(m.add_months(1)),
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
            PeriodType::Week => {
                *self = Period::Week(start.date_naive().week(Weekday::Sun).first_day())
            }
            PeriodType::Month => *self = Period::Month(start.date_naive().with_day(1).unwrap()),
        }
    }
}

// Extensions for convenience

pub trait NaiveDateExt {
    fn at_midnight(&self) -> NaiveDateTime;
    fn prev(self) -> NaiveDate;
    fn next(self) -> NaiveDate;
    fn add_days(self, days: u64) -> NaiveDate;
    fn sub_days(self, days: u64) -> NaiveDate;
    fn add_months(self, months: u32) -> NaiveDate;
    fn sub_months(self, months: u32) -> NaiveDate;
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

    fn add_days(self, days: u64) -> NaiveDate {
        self.checked_add_days(Days::new(days)).unwrap()
    }

    fn sub_days(self, days: u64) -> NaiveDate {
        self.checked_sub_days(Days::new(days)).unwrap()
    }

    fn add_months(self, months: u32) -> NaiveDate {
        self.checked_add_months(Months::new(months)).unwrap()
    }

    fn sub_months(self, months: u32) -> NaiveDate {
        self.checked_sub_months(Months::new(months)).unwrap()
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
