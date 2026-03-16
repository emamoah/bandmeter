use chrono::Utc;
use gpui::SharedString;
use gpui_component::select::SelectItem;

pub const SECS_MIN: i64 = 60;
pub const SECS_HOUR: i64 = SECS_MIN * 60;
pub const SECS_DAY: i64 = SECS_HOUR * 24;

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
    Hour(i64),
    Day(i64),
}

impl Period {
    pub fn current(period_type: &PeriodType) -> Self {
        let now = Utc::now().timestamp();
        match period_type {
            PeriodType::Hour => Period::Hour(now - now % SECS_HOUR),
            PeriodType::Day => Period::Day(now - now % SECS_DAY),
        }
    }

    pub fn is_current(&self) -> bool {
        Utc::now().timestamp() < self.bounds().1
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

    pub fn prev(&mut self) -> bool {
        match *self {
            // Unchecked arithmetic because it's *impractical* to under/overflow
            Period::Hour(h) => *self = Period::Hour(h - SECS_HOUR),
            Period::Day(d) => *self = Period::Day(d - SECS_DAY),
        }

        true
    }

    pub fn next(&mut self) -> bool {
        if self.is_current() {
            return false;
        }

        match *self {
            Period::Hour(h) => *self = Period::Hour(h + SECS_HOUR),
            Period::Day(d) => *self = Period::Day(d + SECS_DAY),
        }

        true
    }

    pub fn switch(&mut self, to: &PeriodType) {
        let (current_start, current_end) = self.bounds();

        if Utc::now().timestamp() < current_end {
            *self = Period::current(to);
            return;
        }

        match to {
            PeriodType::Hour => *self = Period::Hour(current_start - current_start % SECS_HOUR),
            PeriodType::Day => *self = Period::Day(current_start - current_start % SECS_DAY),
        }
    }
}
