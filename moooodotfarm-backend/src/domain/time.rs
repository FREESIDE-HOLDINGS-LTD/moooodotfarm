use crate::errors::Result;
use anyhow::anyhow;
use chrono::TimeZone as _;
use chrono::{Datelike as _, DurationRound};
use std::fmt::Display;
use std::ops::{Add, AddAssign, Sub};

pub enum TimeZone {
    UTC,
    Local,
    OffsetEastInSeconds(u64),
    OffsetWestInSeconds(u64),
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub struct NaiveDate {
    nd: chrono::NaiveDate,
}

impl NaiveDate {
    pub fn new_from_ydy(year: i32, day_of_the_year: u32) -> Result<Self> {
        let nd = chrono::NaiveDate::from_yo_opt(year, day_of_the_year)
            .ok_or_else(|| anyhow!("error creating naive date"))?;
        Ok(Self { nd })
    }

    pub fn day(&self) -> u32 {
        self.nd.day()
    }

    pub fn month(&self) -> u32 {
        self.nd.month()
    }

    pub fn year(&self) -> i32 {
        self.nd.year()
    }
}

impl AddAssign<&Duration> for NaiveDate {
    fn add_assign(&mut self, rhs: &Duration) {
        self.nd += rhs.d;
    }
}

impl AddAssign<Duration> for NaiveDate {
    fn add_assign(&mut self, rhs: Duration) {
        self.nd += rhs.d;
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub struct NaiveDateTime {
    ndt: chrono::NaiveDateTime,
}

impl NaiveDateTime {
    pub fn new_from_str(s: &str, format: &str) -> Result<Self> {
        let ndt = chrono::NaiveDateTime::parse_from_str(s, format)?;
        Ok(Self { ndt })
    }

    pub fn new_from_ymdhms(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Self {
        let date = chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let time = chrono::NaiveTime::from_hms_opt(hour, min, sec).unwrap();
        let ndt = chrono::NaiveDateTime::new(date, time);
        Self { ndt }
    }

    pub fn attach_timezone(&self, timezone: TimeZone) -> DateTime {
        match timezone {
            TimeZone::UTC => {
                let dt = chrono::Utc.from_utc_datetime(&self.ndt);
                DateTime::new(dt.fixed_offset())
            }
            TimeZone::Local => {
                let dt = chrono::Local
                    .from_local_datetime(&self.ndt)
                    .single()
                    .unwrap();
                DateTime::new(dt.fixed_offset())
            }
            TimeZone::OffsetEastInSeconds(seconds_east) => {
                let dt = chrono::FixedOffset::east_opt(seconds_east as i32)
                    .unwrap()
                    .from_local_datetime(&self.ndt)
                    .unwrap();
                DateTime::new(dt)
            }
            TimeZone::OffsetWestInSeconds(seconds_west) => {
                let dt = chrono::FixedOffset::west_opt(seconds_west as i32)
                    .unwrap()
                    .from_local_datetime(&self.ndt)
                    .unwrap();
                DateTime::new(dt)
            }
        }
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub struct DateTime {
    dt: chrono::DateTime<chrono::FixedOffset>,
}

impl DateTime {
    pub fn now() -> Self {
        Self {
            dt: chrono::Utc::now().fixed_offset(),
        }
    }

    pub fn new_from_unix_timestamp(unix_timestamp: u64) -> Self {
        let dt = chrono::DateTime::from_timestamp(unix_timestamp as i64, 0).unwrap();
        Self {
            dt: dt.fixed_offset(),
        }
    }

    pub fn new_from_str(s: &str, format: &str) -> Result<Self> {
        let dt = chrono::DateTime::parse_from_str(s, format)?;
        Ok(Self { dt })
    }

    fn new(dt: chrono::DateTime<chrono::FixedOffset>) -> Self {
        Self { dt }
    }

    pub fn in_timezone(&self, timezone: TimeZone) -> DateTime {
        let dt = match timezone {
            TimeZone::UTC => self.dt.with_timezone(&chrono::Utc).fixed_offset(),
            TimeZone::Local => self.dt.with_timezone(&chrono::Local).fixed_offset(),
            TimeZone::OffsetEastInSeconds(seconds_east) => self
                .dt
                .with_timezone(&chrono::FixedOffset::east_opt(seconds_east as i32).unwrap()),
            TimeZone::OffsetWestInSeconds(seconds_west) => self
                .dt
                .with_timezone(&chrono::FixedOffset::west_opt(seconds_west as i32).unwrap()),
        };
        DateTime::new(dt)
    }

    pub fn truncate_to_seconds(&self) -> Result<Self> {
        Ok(Self::new(
            self.dt.duration_trunc(chrono::Duration::seconds(1))?,
        ))
    }

    pub fn day(&self) -> u32 {
        self.dt.day()
    }

    pub fn month(&self) -> u32 {
        self.dt.month()
    }

    pub fn year(&self) -> i32 {
        self.dt.year()
    }

    pub fn format(&self, format: &str) -> String {
        self.dt.format(format).to_string()
    }

    pub fn ago(&self) -> String {
        let now = DateTime::now();
        let duration = now - self;
        let std_duration = duration
            .d
            .to_std()
            .unwrap_or(std::time::Duration::from_secs(0));
        timeago::Formatter::new().convert(std_duration)
    }
}

impl Add<&Duration> for DateTime {
    type Output = DateTime;

    fn add(self, rhs: &Duration) -> Self::Output {
        DateTime::new(self.dt + rhs.d)
    }
}

impl Add<&Duration> for &DateTime {
    type Output = DateTime;

    fn add(self, rhs: &Duration) -> Self::Output {
        DateTime::new(self.dt + rhs.d)
    }
}

impl Sub<&Duration> for DateTime {
    type Output = DateTime;

    fn sub(self, rhs: &Duration) -> Self::Output {
        DateTime::new(self.dt - rhs.d)
    }
}

impl Sub<&Duration> for &DateTime {
    type Output = DateTime;

    fn sub(self, rhs: &Duration) -> Self::Output {
        DateTime::new(self.dt - rhs.d)
    }
}

impl Add<Duration> for DateTime {
    type Output = DateTime;

    fn add(self, rhs: Duration) -> Self::Output {
        DateTime::new(self.dt + rhs.d)
    }
}

impl Add<Duration> for &DateTime {
    type Output = DateTime;

    fn add(self, rhs: Duration) -> Self::Output {
        DateTime::new(self.dt + rhs.d)
    }
}

impl Sub<Duration> for DateTime {
    type Output = DateTime;

    fn sub(self, rhs: Duration) -> Self::Output {
        DateTime::new(self.dt - rhs.d)
    }
}

impl Sub<Duration> for &DateTime {
    type Output = DateTime;

    fn sub(self, rhs: Duration) -> Self::Output {
        DateTime::new(self.dt - rhs.d)
    }
}

impl Sub<&DateTime> for DateTime {
    type Output = Duration;

    fn sub(self, rhs: &DateTime) -> Self::Output {
        Duration::new(self.dt - rhs.dt)
    }
}

impl Sub<&DateTime> for &DateTime {
    type Output = Duration;

    fn sub(self, rhs: &DateTime) -> Self::Output {
        Duration::new(self.dt - rhs.dt)
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dt.format("%Y-%m-%d %H:%M:%S %z"))
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub struct Duration {
    d: chrono::Duration,
}

impl Duration {
    pub fn new_from_seconds(seconds: u64) -> Self {
        Self {
            d: chrono::Duration::new(seconds as i64, 0).unwrap(),
        }
    }

    pub fn new_from_minutes(minutes: u64) -> Self {
        Self {
            d: chrono::Duration::minutes(minutes as i64),
        }
    }

    pub fn new_from_hours(hours: u64) -> Self {
        Self {
            d: chrono::Duration::hours(hours as i64),
        }
    }

    pub fn new_from_days(days: u64) -> Self {
        Self {
            d: chrono::Duration::days(days as i64),
        }
    }

    pub fn new_from_std(d: std::time::Duration) -> Self {
        Self {
            d: chrono::Duration::from_std(d).unwrap(),
        }
    }

    fn new(d: chrono::Duration) -> Self {
        Self { d }
    }

    pub fn as_seconds(&self) -> f64 {
        self.d.as_seconds_f64()
    }
}
