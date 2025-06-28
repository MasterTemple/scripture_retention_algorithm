#![allow(unused)]

pub type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

use std::borrow::Cow;

use chrono::NaiveDate;
use itertools::Itertools;

fn split_into_n_parts<T: Clone>(vec: Vec<T>, n: usize) -> Vec<Vec<T>> {
    let len = vec.len();
    let base = len / n;
    let remainder = len % n;

    let mut result = Vec::with_capacity(n);
    let mut start = 0;

    for i in 0..n {
        let extra = if i < remainder { 1 } else { 0 };
        let end = start + base + extra;
        result.push(vec[start..end].to_vec());
        start = end;
    }

    result
}

#[derive(Debug)]
pub struct VerseEntry {
    date: NaiveDate,
    reference: String,
}

const FMT: &'static str = "%Y-%m-%d";

impl VerseEntry {
    pub fn new(date: &str, reference: impl Into<String>) -> AnyResult<Self> {
        let reference = reference.into();
        let date = NaiveDate::parse_from_str(date, FMT)?;
        Ok(Self { date, reference })
    }

    pub fn weeks_in(&self, today: NaiveDate) -> i64 {
        (today - self.date).num_weeks()
    }

    pub fn frequency(&self, today: NaiveDate) -> Frequency {
        Frequency::new(self.weeks_in(today))
    }

    pub fn calculate_relative(&self, today: NaiveDate) -> Verse {
        let weeks_in = self.weeks_in(today);
        Verse {
            weeks_in,
            reference: Cow::Owned(self.reference.clone()),
        }
    }
}

#[derive(Debug)]
pub struct VerseList {
    today: NaiveDate,
    references: Vec<VerseEntry>,
}

impl VerseList {
    pub fn new(date: &str, references: Vec<VerseEntry>) -> AnyResult<Self> {
        let today = NaiveDate::parse_from_str(date, FMT)?;
        Ok(Self { today, references })
    }

    pub fn relative_verses(&self) -> Vec<Verse> {
        self.references
            .iter()
            .map(|verse| verse.calculate_relative(self.today))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Verse<'a> {
    weeks_in: i64,
    reference: Cow<'a, String>,
}

impl<'a> Verse<'a> {
    pub fn frequency(&self) -> Frequency {
        Frequency::new(self.weeks_in)
    }

    pub fn add_offset(&mut self, weeks: i64) {
        self.weeks_in += weeks;
    }

    pub fn with_offset(&self, weeks: i64) -> Self {
        let mut it = self.clone();
        it.weeks_in += weeks;
        it
    }

    pub fn is_daily(&self) -> bool {
        self.frequency() == Frequency::Daily
    }

    pub fn is_weekly(&self) -> bool {
        self.frequency() == Frequency::Weekly
    }

    /**
    I am not doing this monthly according to a calendar.
    Monthly means once every 4 weeks.
    This allows for an implementation that checks if:
    1. This verse should be recited monthly
    2. This is every 4th week
    */
    pub fn is_monthly_this_week(&self) -> bool {
        let is_monthly = self.frequency() == Frequency::Monthly;
        let is_monthly_this_week = self.weeks_in % 4 == 1;
        is_monthly && is_monthly_this_week
    }

    pub fn is_monthly_week(&self, n: i64) -> bool {
        let is_monthly = self.frequency() == Frequency::Monthly;
        let is_monthly_this_week = self.weeks_in % 4 == n;
        is_monthly && is_monthly_this_week
    }
}

pub struct RelativeVerseList {}

#[derive(PartialEq, Debug)]
pub enum Frequency {
    NotStarted,
    Daily,
    Weekly,
    Monthly,
    Done,
}

impl Frequency {
    pub fn new(weeks_in: i64) -> Self {
        if weeks_in < 0 {
            Frequency::NotStarted
        } else if weeks_in < 7 {
            Frequency::Daily
        } else if weeks_in < 7 + 28 {
            Frequency::Weekly
        } else if weeks_in < 7 + 28 + 336 {
            Frequency::Monthly
        } else {
            Frequency::Done
        }
    }
}

#[derive(Debug)]
pub struct VersesForADay<'a> {
    daily: Vec<Verse<'a>>,
    weekly: Vec<Verse<'a>>,
    monthly: Vec<Verse<'a>>,
}

#[derive(Debug)]
pub struct VersesForAWeek<'a> {
    days: Vec<VersesForADay<'a>>,
}

impl<'a> VersesForAWeek<'a> {
    pub fn new<'b>(verses: &'b Vec<Verse<'a>>, n: i64) -> Self {
        let daily: Vec<_> = verses
            .iter()
            .filter(|verse| verse.is_daily())
            .cloned()
            .collect();
        let weekly: Vec<_> = verses
            .iter()
            .filter(|verse| verse.is_weekly())
            .cloned()
            .collect();
        let monthly: Vec<_> = verses
            .iter()
            .filter(|verse| verse.is_monthly_week(n))
            .cloned()
            .collect();
        let weekly = split_into_n_parts(weekly, 7);
        let monthly = split_into_n_parts(monthly, 7);
        let days = weekly
            .into_iter()
            .zip(monthly)
            .map(|(weekly, monthly)| VersesForADay {
                daily: daily.clone(),
                weekly,
                monthly,
            })
            .collect_vec();
        Self { days }
    }
}

#[derive(Debug)]
pub struct VersesForAMonth<'a> {
    weeks: Vec<VersesForAWeek<'a>>,
}

impl<'a> VersesForAMonth<'a> {
    pub fn new(verses: &'a Vec<Verse>) -> Self {
        let weeks = (0..=3)
            .map(|n| {
                // perhaps [`VersesForAWeek::new`] should do the offset
                let verses = verses.iter().map(|v| v.with_offset(n)).collect_vec();
                VersesForAWeek::new(&verses, n)
            })
            .collect_vec();
        Self { weeks }
    }

    pub fn stats(&self) -> String {
        self.weeks
            .iter()
            .map(|week| {
                week.days
                    .iter()
                    .map(|day| {
                        format!(
                            "D: {} | W: {} | M: {}",
                            day.daily.len(),
                            day.weekly.len(),
                            day.monthly.len()
                        )
                    })
                    .join("\n")
            })
            .join("\n---\n")
    }
}

fn main() -> AnyResult<()> {
    // let date = "2025-07-06";
    let date = "2033-02-06";

    let references = vec![
        VerseEntry::new("2025-07-06", "John 1:1")?,
        VerseEntry::new("2025-07-13", "John 1:2")?,
        VerseEntry::new("2025-07-20", "John 1:3")?,
        VerseEntry::new("2025-07-27", "John 1:4")?,
        VerseEntry::new("2025-08-03", "John 1:5")?,
        VerseEntry::new("2025-08-10", "John 1:6")?,
        VerseEntry::new("2025-08-17", "John 1:7")?,
        VerseEntry::new("2025-08-24", "John 1:8")?,
        VerseEntry::new("2025-08-31", "John 1:9")?,
        VerseEntry::new("2025-09-07", "John 1:10")?,
        VerseEntry::new("2025-09-14", "John 1:11")?,
        VerseEntry::new("2025-09-21", "John 1:12")?,
        VerseEntry::new("2025-09-28", "John 1:13")?,
        VerseEntry::new("2025-10-05", "John 1:14")?,
        VerseEntry::new("2025-10-12", "John 1:15")?,
        VerseEntry::new("2025-10-19", "John 1:16")?,
        VerseEntry::new("2025-10-26", "John 1:17")?,
        VerseEntry::new("2025-11-02", "John 1:18")?,
        VerseEntry::new("2025-11-09", "John 1:19")?,
        VerseEntry::new("2025-11-16", "John 1:20")?,
        VerseEntry::new("2025-11-23", "John 1:21")?,
        VerseEntry::new("2025-11-30", "John 1:22")?,
        VerseEntry::new("2025-12-07", "John 1:23")?,
        VerseEntry::new("2025-12-14", "John 1:24")?,
        VerseEntry::new("2025-12-21", "John 1:25")?,
        VerseEntry::new("2025-12-28", "John 1:26")?,
        VerseEntry::new("2026-01-04", "John 1:27")?,
        VerseEntry::new("2026-01-11", "John 1:28")?,
        VerseEntry::new("2026-01-18", "John 1:29")?,
        VerseEntry::new("2026-01-25", "John 1:30")?,
        VerseEntry::new("2026-02-01", "John 1:31")?,
        VerseEntry::new("2026-02-08", "John 1:32")?,
        VerseEntry::new("2026-02-15", "John 1:33")?,
        VerseEntry::new("2026-02-22", "John 1:34")?,
        VerseEntry::new("2026-03-01", "John 1:35")?,
        VerseEntry::new("2026-03-08", "John 1:36")?,
        VerseEntry::new("2026-03-15", "John 1:37")?,
        VerseEntry::new("2026-03-22", "John 1:38")?,
        VerseEntry::new("2026-03-29", "John 1:39")?,
        VerseEntry::new("2026-04-05", "John 1:40")?,
        VerseEntry::new("2026-04-12", "John 1:41")?,
        VerseEntry::new("2026-04-19", "John 1:42")?,
        VerseEntry::new("2026-04-26", "John 1:43")?,
        VerseEntry::new("2026-05-03", "John 1:44")?,
        VerseEntry::new("2026-05-10", "John 1:45")?,
        VerseEntry::new("2026-05-17", "John 1:46")?,
        VerseEntry::new("2026-05-24", "John 1:47")?,
        VerseEntry::new("2026-05-31", "John 1:48")?,
        VerseEntry::new("2026-06-07", "John 1:49")?,
        VerseEntry::new("2026-06-14", "John 1:50")?,
        VerseEntry::new("2026-06-21", "John 1:51")?,
        VerseEntry::new("2026-06-28", "John 2:1")?,
        VerseEntry::new("2026-07-05", "John 2:2")?,
        VerseEntry::new("2026-07-12", "John 2:3")?,
        VerseEntry::new("2026-07-19", "John 2:4")?,
        VerseEntry::new("2026-07-26", "John 2:5")?,
        VerseEntry::new("2026-08-02", "John 2:6")?,
        VerseEntry::new("2026-08-09", "John 2:7")?,
        VerseEntry::new("2026-08-16", "John 2:8")?,
        VerseEntry::new("2026-08-23", "John 2:9")?,
        VerseEntry::new("2026-08-30", "John 2:10")?,
        VerseEntry::new("2026-09-06", "John 2:11")?,
        VerseEntry::new("2026-09-13", "John 2:12")?,
        VerseEntry::new("2026-09-20", "John 2:13")?,
        VerseEntry::new("2026-09-27", "John 2:14")?,
        VerseEntry::new("2026-10-04", "John 2:15")?,
        VerseEntry::new("2026-10-11", "John 2:16")?,
        VerseEntry::new("2026-10-18", "John 2:17")?,
        VerseEntry::new("2026-10-25", "John 2:18")?,
        VerseEntry::new("2026-11-01", "John 2:19")?,
        VerseEntry::new("2026-11-08", "John 2:20")?,
        VerseEntry::new("2026-11-15", "John 2:21")?,
        VerseEntry::new("2026-11-22", "John 2:22")?,
        VerseEntry::new("2026-11-29", "John 2:23")?,
        VerseEntry::new("2026-12-06", "John 2:24")?,
        VerseEntry::new("2026-12-13", "John 2:25")?,
        VerseEntry::new("2026-12-20", "John 3:1")?,
        VerseEntry::new("2026-12-27", "John 3:2")?,
        VerseEntry::new("2027-01-03", "John 3:3")?,
        VerseEntry::new("2027-01-10", "John 3:4")?,
        VerseEntry::new("2027-01-17", "John 3:5")?,
        VerseEntry::new("2027-01-24", "John 3:6")?,
        VerseEntry::new("2027-01-31", "John 3:7")?,
        VerseEntry::new("2027-02-07", "John 3:8")?,
        VerseEntry::new("2027-02-14", "John 3:9")?,
        VerseEntry::new("2027-02-21", "John 3:10")?,
        VerseEntry::new("2027-02-28", "John 3:11")?,
        VerseEntry::new("2027-03-07", "John 3:12")?,
        VerseEntry::new("2027-03-14", "John 3:13")?,
        VerseEntry::new("2027-03-21", "John 3:14")?,
        VerseEntry::new("2027-03-28", "John 3:15")?,
        VerseEntry::new("2027-04-04", "John 3:16")?,
        VerseEntry::new("2027-04-11", "John 3:17")?,
        VerseEntry::new("2027-04-18", "John 3:18")?,
        VerseEntry::new("2027-04-25", "John 3:19")?,
        VerseEntry::new("2027-05-02", "John 3:20")?,
        VerseEntry::new("2027-05-09", "John 3:21")?,
        VerseEntry::new("2027-05-16", "John 3:22")?,
        VerseEntry::new("2027-05-23", "John 3:23")?,
        VerseEntry::new("2027-05-30", "John 3:24")?,
        VerseEntry::new("2027-06-06", "John 3:25")?,
        VerseEntry::new("2027-06-13", "John 3:26")?,
        VerseEntry::new("2027-06-20", "John 3:27")?,
        VerseEntry::new("2027-06-27", "John 3:28")?,
        VerseEntry::new("2027-07-04", "John 3:29")?,
        VerseEntry::new("2027-07-11", "John 3:30")?,
        VerseEntry::new("2027-07-18", "John 3:31")?,
        VerseEntry::new("2027-07-25", "John 3:32")?,
        VerseEntry::new("2027-08-01", "John 3:33")?,
        VerseEntry::new("2027-08-08", "John 3:34")?,
        VerseEntry::new("2027-08-15", "John 3:35")?,
        VerseEntry::new("2027-08-22", "John 3:36")?,
        VerseEntry::new("2027-08-29", "John 4:1")?,
        VerseEntry::new("2027-09-05", "John 4:2")?,
        VerseEntry::new("2027-09-12", "John 4:3")?,
        VerseEntry::new("2027-09-19", "John 4:4")?,
        VerseEntry::new("2027-09-26", "John 4:5")?,
        VerseEntry::new("2027-10-03", "John 4:6")?,
        VerseEntry::new("2027-10-10", "John 4:7")?,
        VerseEntry::new("2027-10-17", "John 4:8")?,
        VerseEntry::new("2027-10-24", "John 4:9")?,
        VerseEntry::new("2027-10-31", "John 4:10")?,
        VerseEntry::new("2027-11-07", "John 4:11")?,
        VerseEntry::new("2027-11-14", "John 4:12")?,
        VerseEntry::new("2027-11-21", "John 4:13")?,
        VerseEntry::new("2027-11-28", "John 4:14")?,
        VerseEntry::new("2027-12-05", "John 4:15")?,
        VerseEntry::new("2027-12-12", "John 4:16")?,
        VerseEntry::new("2027-12-19", "John 4:17")?,
        VerseEntry::new("2027-12-26", "John 4:18")?,
        VerseEntry::new("2028-01-02", "John 4:19")?,
        VerseEntry::new("2028-01-09", "John 4:20")?,
        VerseEntry::new("2028-01-16", "John 4:21")?,
        VerseEntry::new("2028-01-23", "John 4:22")?,
        VerseEntry::new("2028-01-30", "John 4:23")?,
        VerseEntry::new("2028-02-06", "John 4:24")?,
        VerseEntry::new("2028-02-13", "John 4:25")?,
        VerseEntry::new("2028-02-20", "John 4:26")?,
        VerseEntry::new("2028-02-27", "John 4:27")?,
        VerseEntry::new("2028-03-05", "John 4:28")?,
        VerseEntry::new("2028-03-12", "John 4:29")?,
        VerseEntry::new("2028-03-19", "John 4:30")?,
        VerseEntry::new("2028-03-26", "John 4:31")?,
        VerseEntry::new("2028-04-02", "John 4:32")?,
        VerseEntry::new("2028-04-09", "John 4:33")?,
        VerseEntry::new("2028-04-16", "John 4:34")?,
        VerseEntry::new("2028-04-23", "John 4:35")?,
        VerseEntry::new("2028-04-30", "John 4:36")?,
        VerseEntry::new("2028-05-07", "John 4:37")?,
        VerseEntry::new("2028-05-14", "John 4:38")?,
        VerseEntry::new("2028-05-21", "John 4:39")?,
        VerseEntry::new("2028-05-28", "John 4:40")?,
        VerseEntry::new("2028-06-04", "John 4:41")?,
        VerseEntry::new("2028-06-11", "John 4:42")?,
        VerseEntry::new("2028-06-18", "John 4:43")?,
        VerseEntry::new("2028-06-25", "John 4:44")?,
        VerseEntry::new("2028-07-02", "John 4:45")?,
        VerseEntry::new("2028-07-09", "John 4:46")?,
        VerseEntry::new("2028-07-16", "John 4:47")?,
        VerseEntry::new("2028-07-23", "John 4:48")?,
        VerseEntry::new("2028-07-30", "John 4:49")?,
        VerseEntry::new("2028-08-06", "John 4:50")?,
        VerseEntry::new("2028-08-13", "John 4:51")?,
        VerseEntry::new("2028-08-20", "John 4:52")?,
        VerseEntry::new("2028-08-27", "John 4:53")?,
        VerseEntry::new("2028-09-03", "John 4:54")?,
        VerseEntry::new("2028-09-10", "John 5:1")?,
        VerseEntry::new("2028-09-17", "John 5:2")?,
        VerseEntry::new("2028-09-24", "John 5:3")?,
        VerseEntry::new("2028-10-01", "John 5:4")?,
        VerseEntry::new("2028-10-08", "John 5:5")?,
        VerseEntry::new("2028-10-15", "John 5:6")?,
        VerseEntry::new("2028-10-22", "John 5:7")?,
        VerseEntry::new("2028-10-29", "John 5:8")?,
        VerseEntry::new("2028-11-05", "John 5:9")?,
        VerseEntry::new("2028-11-12", "John 5:10")?,
        VerseEntry::new("2028-11-19", "John 5:11")?,
        VerseEntry::new("2028-11-26", "John 5:12")?,
        VerseEntry::new("2028-12-03", "John 5:13")?,
        VerseEntry::new("2028-12-10", "John 5:14")?,
        VerseEntry::new("2028-12-17", "John 5:15")?,
        VerseEntry::new("2028-12-24", "John 5:16")?,
        VerseEntry::new("2028-12-31", "John 5:17")?,
        VerseEntry::new("2029-01-07", "John 5:18")?,
        VerseEntry::new("2029-01-14", "John 5:19")?,
        VerseEntry::new("2029-01-21", "John 5:20")?,
        VerseEntry::new("2029-01-28", "John 5:21")?,
        VerseEntry::new("2029-02-04", "John 5:22")?,
        VerseEntry::new("2029-02-11", "John 5:23")?,
        VerseEntry::new("2029-02-18", "John 5:24")?,
        VerseEntry::new("2029-02-25", "John 5:25")?,
        VerseEntry::new("2029-03-04", "John 5:26")?,
        VerseEntry::new("2029-03-11", "John 5:27")?,
        VerseEntry::new("2029-03-18", "John 5:28")?,
        VerseEntry::new("2029-03-25", "John 5:29")?,
        VerseEntry::new("2029-04-01", "John 5:30")?,
        VerseEntry::new("2029-04-08", "John 5:31")?,
        VerseEntry::new("2029-04-15", "John 5:32")?,
        VerseEntry::new("2029-04-22", "John 5:33")?,
        VerseEntry::new("2029-04-29", "John 5:34")?,
        VerseEntry::new("2029-05-06", "John 5:35")?,
        VerseEntry::new("2029-05-13", "John 5:36")?,
        VerseEntry::new("2029-05-20", "John 5:37")?,
        VerseEntry::new("2029-05-27", "John 5:38")?,
        VerseEntry::new("2029-06-03", "John 5:39")?,
        VerseEntry::new("2029-06-10", "John 5:40")?,
        VerseEntry::new("2029-06-17", "John 5:41")?,
        VerseEntry::new("2029-06-24", "John 5:42")?,
        VerseEntry::new("2029-07-01", "John 5:43")?,
        VerseEntry::new("2029-07-08", "John 5:44")?,
        VerseEntry::new("2029-07-15", "John 5:45")?,
        VerseEntry::new("2029-07-22", "John 5:46")?,
        VerseEntry::new("2029-07-29", "John 5:47")?,
        VerseEntry::new("2029-08-05", "John 6:1")?,
        VerseEntry::new("2029-08-12", "John 6:2")?,
        VerseEntry::new("2029-08-19", "John 6:3")?,
        VerseEntry::new("2029-08-26", "John 6:4")?,
        VerseEntry::new("2029-09-02", "John 6:5")?,
        VerseEntry::new("2029-09-09", "John 6:6")?,
        VerseEntry::new("2029-09-16", "John 6:7")?,
        VerseEntry::new("2029-09-23", "John 6:8")?,
        VerseEntry::new("2029-09-30", "John 6:9")?,
        VerseEntry::new("2029-10-07", "John 6:10")?,
        VerseEntry::new("2029-10-14", "John 6:11")?,
        VerseEntry::new("2029-10-21", "John 6:12")?,
        VerseEntry::new("2029-10-28", "John 6:13")?,
        VerseEntry::new("2029-11-04", "John 6:14")?,
        VerseEntry::new("2029-11-11", "John 6:15")?,
        VerseEntry::new("2029-11-18", "John 6:16")?,
        VerseEntry::new("2029-11-25", "John 6:17")?,
        VerseEntry::new("2029-12-02", "John 6:18")?,
        VerseEntry::new("2029-12-09", "John 6:19")?,
        VerseEntry::new("2029-12-16", "John 6:20")?,
        VerseEntry::new("2029-12-23", "John 6:21")?,
        VerseEntry::new("2029-12-30", "John 6:22")?,
        VerseEntry::new("2030-01-06", "John 6:23")?,
        VerseEntry::new("2030-01-13", "John 6:24")?,
        VerseEntry::new("2030-01-20", "John 6:25")?,
        VerseEntry::new("2030-01-27", "John 6:26")?,
        VerseEntry::new("2030-02-03", "John 6:27")?,
        VerseEntry::new("2030-02-10", "John 6:28")?,
        VerseEntry::new("2030-02-17", "John 6:29")?,
        VerseEntry::new("2030-02-24", "John 6:30")?,
        VerseEntry::new("2030-03-03", "John 6:31")?,
        VerseEntry::new("2030-03-10", "John 6:32")?,
        VerseEntry::new("2030-03-17", "John 6:33")?,
        VerseEntry::new("2030-03-24", "John 6:34")?,
        VerseEntry::new("2030-03-31", "John 6:35")?,
        VerseEntry::new("2030-04-07", "John 6:36")?,
        VerseEntry::new("2030-04-14", "John 6:37")?,
        VerseEntry::new("2030-04-21", "John 6:38")?,
        VerseEntry::new("2030-04-28", "John 6:39")?,
        VerseEntry::new("2030-05-05", "John 6:40")?,
        VerseEntry::new("2030-05-12", "John 6:41")?,
        VerseEntry::new("2030-05-19", "John 6:42")?,
        VerseEntry::new("2030-05-26", "John 6:43")?,
        VerseEntry::new("2030-06-02", "John 6:44")?,
        VerseEntry::new("2030-06-09", "John 6:45")?,
        VerseEntry::new("2030-06-16", "John 6:46")?,
        VerseEntry::new("2030-06-23", "John 6:47")?,
        VerseEntry::new("2030-06-30", "John 6:48")?,
        VerseEntry::new("2030-07-07", "John 6:49")?,
        VerseEntry::new("2030-07-14", "John 6:50")?,
        VerseEntry::new("2030-07-21", "John 6:51")?,
        VerseEntry::new("2030-07-28", "John 6:52")?,
        VerseEntry::new("2030-08-04", "John 6:53")?,
        VerseEntry::new("2030-08-11", "John 6:54")?,
        VerseEntry::new("2030-08-18", "John 6:55")?,
        VerseEntry::new("2030-08-25", "John 6:56")?,
        VerseEntry::new("2030-09-01", "John 6:57")?,
        VerseEntry::new("2030-09-08", "John 6:58")?,
        VerseEntry::new("2030-09-15", "John 6:59")?,
        VerseEntry::new("2030-09-22", "John 6:60")?,
        VerseEntry::new("2030-09-29", "John 6:61")?,
        VerseEntry::new("2030-10-06", "John 6:62")?,
        VerseEntry::new("2030-10-13", "John 6:63")?,
        VerseEntry::new("2030-10-20", "John 6:64")?,
        VerseEntry::new("2030-10-27", "John 6:65")?,
        VerseEntry::new("2030-11-03", "John 6:66")?,
        VerseEntry::new("2030-11-10", "John 6:67")?,
        VerseEntry::new("2030-11-17", "John 6:68")?,
        VerseEntry::new("2030-11-24", "John 6:69")?,
        VerseEntry::new("2030-12-01", "John 6:70")?,
        VerseEntry::new("2030-12-08", "John 6:71")?,
        VerseEntry::new("2030-12-15", "John 7:1")?,
        VerseEntry::new("2030-12-22", "John 7:2")?,
        VerseEntry::new("2030-12-29", "John 7:3")?,
        VerseEntry::new("2031-01-05", "John 7:4")?,
        VerseEntry::new("2031-01-12", "John 7:5")?,
        VerseEntry::new("2031-01-19", "John 7:6")?,
        VerseEntry::new("2031-01-26", "John 7:7")?,
        VerseEntry::new("2031-02-02", "John 7:8")?,
        VerseEntry::new("2031-02-09", "John 7:9")?,
        VerseEntry::new("2031-02-16", "John 7:10")?,
        VerseEntry::new("2031-02-23", "John 7:11")?,
        VerseEntry::new("2031-03-02", "John 7:12")?,
        VerseEntry::new("2031-03-09", "John 7:13")?,
        VerseEntry::new("2031-03-16", "John 7:14")?,
        VerseEntry::new("2031-03-23", "John 7:15")?,
        VerseEntry::new("2031-03-30", "John 7:16")?,
        VerseEntry::new("2031-04-06", "John 7:17")?,
        VerseEntry::new("2031-04-13", "John 7:18")?,
        VerseEntry::new("2031-04-20", "John 7:19")?,
        VerseEntry::new("2031-04-27", "John 7:20")?,
        VerseEntry::new("2031-05-04", "John 7:21")?,
        VerseEntry::new("2031-05-11", "John 7:22")?,
        VerseEntry::new("2031-05-18", "John 7:23")?,
        VerseEntry::new("2031-05-25", "John 7:24")?,
        VerseEntry::new("2031-06-01", "John 7:25")?,
        VerseEntry::new("2031-06-08", "John 7:26")?,
        VerseEntry::new("2031-06-15", "John 7:27")?,
        VerseEntry::new("2031-06-22", "John 7:28")?,
        VerseEntry::new("2031-06-29", "John 7:29")?,
        VerseEntry::new("2031-07-06", "John 7:30")?,
        VerseEntry::new("2031-07-13", "John 7:31")?,
        VerseEntry::new("2031-07-20", "John 7:32")?,
        VerseEntry::new("2031-07-27", "John 7:33")?,
        VerseEntry::new("2031-08-03", "John 7:34")?,
        VerseEntry::new("2031-08-10", "John 7:35")?,
        VerseEntry::new("2031-08-17", "John 7:36")?,
        VerseEntry::new("2031-08-24", "John 7:37")?,
        VerseEntry::new("2031-08-31", "John 7:38")?,
        VerseEntry::new("2031-09-07", "John 7:39")?,
        VerseEntry::new("2031-09-14", "John 7:40")?,
        VerseEntry::new("2031-09-21", "John 7:41")?,
        VerseEntry::new("2031-09-28", "John 7:42")?,
        VerseEntry::new("2031-10-05", "John 7:43")?,
        VerseEntry::new("2031-10-12", "John 7:44")?,
        VerseEntry::new("2031-10-19", "John 7:45")?,
        VerseEntry::new("2031-10-26", "John 7:46")?,
        VerseEntry::new("2031-11-02", "John 7:47")?,
        VerseEntry::new("2031-11-09", "John 7:48")?,
        VerseEntry::new("2031-11-16", "John 7:49")?,
        VerseEntry::new("2031-11-23", "John 7:50")?,
        VerseEntry::new("2031-11-30", "John 7:51")?,
        VerseEntry::new("2031-12-07", "John 7:52")?,
        VerseEntry::new("2031-12-14", "John 7:53")?,
        VerseEntry::new("2031-12-21", "John 8:1")?,
        VerseEntry::new("2031-12-28", "John 8:2")?,
        VerseEntry::new("2032-01-04", "John 8:3")?,
        VerseEntry::new("2032-01-11", "John 8:4")?,
        VerseEntry::new("2032-01-18", "John 8:5")?,
        VerseEntry::new("2032-01-25", "John 8:6")?,
        VerseEntry::new("2032-02-01", "John 8:7")?,
        VerseEntry::new("2032-02-08", "John 8:8")?,
        VerseEntry::new("2032-02-15", "John 8:9")?,
        VerseEntry::new("2032-02-22", "John 8:10")?,
        VerseEntry::new("2032-02-29", "John 8:11")?,
        VerseEntry::new("2032-03-07", "John 8:12")?,
        VerseEntry::new("2032-03-14", "John 8:13")?,
        VerseEntry::new("2032-03-21", "John 8:14")?,
        VerseEntry::new("2032-03-28", "John 8:15")?,
        VerseEntry::new("2032-04-04", "John 8:16")?,
        VerseEntry::new("2032-04-11", "John 8:17")?,
        VerseEntry::new("2032-04-18", "John 8:18")?,
        VerseEntry::new("2032-04-25", "John 8:19")?,
        VerseEntry::new("2032-05-02", "John 8:20")?,
        VerseEntry::new("2032-05-09", "John 8:21")?,
        VerseEntry::new("2032-05-16", "John 8:22")?,
        VerseEntry::new("2032-05-23", "John 8:23")?,
        VerseEntry::new("2032-05-30", "John 8:24")?,
        VerseEntry::new("2032-06-06", "John 8:25")?,
        VerseEntry::new("2032-06-13", "John 8:26")?,
        VerseEntry::new("2032-06-20", "John 8:27")?,
        VerseEntry::new("2032-06-27", "John 8:28")?,
        VerseEntry::new("2032-07-04", "John 8:29")?,
        VerseEntry::new("2032-07-11", "John 8:30")?,
        VerseEntry::new("2032-07-18", "John 8:31")?,
        VerseEntry::new("2032-07-25", "John 8:32")?,
        VerseEntry::new("2032-08-01", "John 8:33")?,
        VerseEntry::new("2032-08-08", "John 8:34")?,
        VerseEntry::new("2032-08-15", "John 8:35")?,
        VerseEntry::new("2032-08-22", "John 8:36")?,
        VerseEntry::new("2032-08-29", "John 8:37")?,
        VerseEntry::new("2032-09-05", "John 8:38")?,
        VerseEntry::new("2032-09-12", "John 8:39")?,
        VerseEntry::new("2032-09-19", "John 8:40")?,
        VerseEntry::new("2032-09-26", "John 8:41")?,
        VerseEntry::new("2032-10-03", "John 8:42")?,
        VerseEntry::new("2032-10-10", "John 8:43")?,
        VerseEntry::new("2032-10-17", "John 8:44")?,
        VerseEntry::new("2032-10-24", "John 8:45")?,
        VerseEntry::new("2032-10-31", "John 8:46")?,
        VerseEntry::new("2032-11-07", "John 8:47")?,
        VerseEntry::new("2032-11-14", "John 8:48")?,
        VerseEntry::new("2032-11-21", "John 8:49")?,
        VerseEntry::new("2032-11-28", "John 8:50")?,
        VerseEntry::new("2032-12-05", "John 8:51")?,
        VerseEntry::new("2032-12-12", "John 8:52")?,
        VerseEntry::new("2032-12-19", "John 8:53")?,
        VerseEntry::new("2032-12-26", "John 8:54")?,
        VerseEntry::new("2033-01-02", "John 8:55")?,
        VerseEntry::new("2033-01-09", "John 8:56")?,
        VerseEntry::new("2033-01-16", "John 8:57")?,
        VerseEntry::new("2033-01-23", "John 8:58")?,
        VerseEntry::new("2033-01-30", "John 8:59")?,
        VerseEntry::new("2033-02-06", "John 9:1")?,
    ];

    let list = VerseList::new(date, references)?;
    let verses = list.relative_verses();

    // dbg!(VersesForAWeek::new(&verses, 1));
    // dbg!(VersesForAMonth::new(&verses));
    println!("{}", VersesForAMonth::new(&verses).stats());

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn idk() -> AnyResult<()> {
//         let date = "2025-07-06";
//         let references = vec![
//             VerseEntry::new("2025-07-06", "John 1:1")?,
//             VerseEntry::new("2025-07-13", "John 1:2")?,
//         ];
//         let list = VerseList::new(date, references)?;
//         let verses = list.relative_verses();
//
//         VersesForAMonth::new(&verses);
//
//         Ok(())
//     }
//
//     // #[test]
//     fn daily() -> AnyResult<()> {
//         let reference = "John 1:1".to_string();
//         let today = NaiveDate::parse_from_str("2025-07-06", FMT)?;
//
//         macro_rules! check_entry {
//             ($date:literal, $freq:ident) => {
//                 assert_eq!(
//                     VerseEntry::new($date, "John 1:1".to_string())?.frequency(today),
//                     Frequency::$freq,
//                 )
//             };
//         }
//
//         check_entry!("2025-07-06", Daily);
//         check_entry!("2025-07-13", Daily);
//         check_entry!("2025-07-20", Daily);
//         check_entry!("2025-07-27", Daily);
//         check_entry!("2025-08-03", Daily);
//         check_entry!("2025-08-10", Daily);
//         check_entry!("2025-08-17", Daily);
//
//         check_entry!("2025-08-24", Weekly);
//         check_entry!("2025-08-31", Weekly);
//         check_entry!("2025-09-07", Weekly);
//         check_entry!("2025-09-14", Weekly);
//         check_entry!("2025-09-21", Weekly);
//         check_entry!("2025-09-28", Weekly);
//         check_entry!("2025-10-05", Weekly);
//         check_entry!("2025-10-12", Weekly);
//         check_entry!("2025-10-19", Weekly);
//         check_entry!("2025-10-26", Weekly);
//         check_entry!("2025-11-02", Weekly);
//         check_entry!("2025-11-09", Weekly);
//         check_entry!("2025-11-16", Weekly);
//         check_entry!("2025-11-23", Weekly);
//         check_entry!("2025-11-30", Weekly);
//         check_entry!("2025-12-07", Weekly);
//         check_entry!("2025-12-14", Weekly);
//         check_entry!("2025-12-21", Weekly);
//         check_entry!("2025-12-28", Weekly);
//         check_entry!("2026-01-04", Weekly);
//         check_entry!("2026-01-11", Weekly);
//         check_entry!("2026-01-18", Weekly);
//         check_entry!("2026-01-25", Weekly);
//         check_entry!("2026-02-01", Weekly);
//         check_entry!("2026-02-08", Weekly);
//         check_entry!("2026-02-15", Weekly);
//         check_entry!("2026-02-22", Weekly);
//         check_entry!("2026-03-01", Weekly);
//
//         check_entry!("2026-03-08", Monthly);
//         check_entry!("2026-03-15", Monthly);
//         check_entry!("2026-03-22", Monthly);
//         check_entry!("2026-03-29", Monthly);
//         check_entry!("2026-04-05", Monthly);
//         check_entry!("2026-04-12", Monthly);
//         check_entry!("2026-04-19", Monthly);
//         check_entry!("2026-04-26", Monthly);
//         check_entry!("2026-05-03", Monthly);
//         check_entry!("2026-05-10", Monthly);
//         check_entry!("2026-05-17", Monthly);
//         check_entry!("2026-05-24", Monthly);
//         check_entry!("2026-05-31", Monthly);
//         check_entry!("2026-06-07", Monthly);
//         check_entry!("2026-06-14", Monthly);
//         check_entry!("2026-06-21", Monthly);
//         check_entry!("2026-06-28", Monthly);
//         check_entry!("2026-07-05", Monthly);
//         check_entry!("2026-07-12", Monthly);
//         check_entry!("2026-07-19", Monthly);
//         check_entry!("2026-07-26", Monthly);
//         check_entry!("2026-08-02", Monthly);
//         check_entry!("2026-08-09", Monthly);
//         check_entry!("2026-08-16", Monthly);
//         check_entry!("2026-08-23", Monthly);
//         check_entry!("2026-08-30", Monthly);
//         check_entry!("2026-09-06", Monthly);
//         check_entry!("2026-09-13", Monthly);
//         check_entry!("2026-09-20", Monthly);
//         check_entry!("2026-09-27", Monthly);
//         check_entry!("2026-10-04", Monthly);
//         check_entry!("2026-10-11", Monthly);
//         check_entry!("2026-10-18", Monthly);
//         check_entry!("2026-10-25", Monthly);
//         check_entry!("2026-11-01", Monthly);
//         check_entry!("2026-11-08", Monthly);
//         check_entry!("2026-11-15", Monthly);
//         check_entry!("2026-11-22", Monthly);
//         check_entry!("2026-11-29", Monthly);
//         check_entry!("2026-12-06", Monthly);
//         check_entry!("2026-12-13", Monthly);
//         check_entry!("2026-12-20", Monthly);
//         check_entry!("2026-12-27", Monthly);
//         check_entry!("2027-01-03", Monthly);
//         check_entry!("2027-01-10", Monthly);
//         check_entry!("2027-01-17", Monthly);
//         check_entry!("2027-01-24", Monthly);
//         check_entry!("2027-01-31", Monthly);
//         check_entry!("2027-02-07", Monthly);
//         check_entry!("2027-02-14", Monthly);
//         check_entry!("2027-02-21", Monthly);
//         check_entry!("2027-02-28", Monthly);
//         check_entry!("2027-03-07", Monthly);
//         check_entry!("2027-03-14", Monthly);
//         check_entry!("2027-03-21", Monthly);
//         check_entry!("2027-03-28", Monthly);
//         check_entry!("2027-04-04", Monthly);
//         check_entry!("2027-04-11", Monthly);
//         check_entry!("2027-04-18", Monthly);
//         check_entry!("2027-04-25", Monthly);
//         check_entry!("2027-05-02", Monthly);
//         check_entry!("2027-05-09", Monthly);
//         check_entry!("2027-05-16", Monthly);
//         check_entry!("2027-05-23", Monthly);
//         check_entry!("2027-05-30", Monthly);
//         check_entry!("2027-06-06", Monthly);
//         check_entry!("2027-06-13", Monthly);
//         check_entry!("2027-06-20", Monthly);
//         check_entry!("2027-06-27", Monthly);
//         check_entry!("2027-07-04", Monthly);
//         check_entry!("2027-07-11", Monthly);
//         check_entry!("2027-07-18", Monthly);
//         check_entry!("2027-07-25", Monthly);
//         check_entry!("2027-08-01", Monthly);
//         check_entry!("2027-08-08", Monthly);
//         check_entry!("2027-08-15", Monthly);
//         check_entry!("2027-08-22", Monthly);
//         check_entry!("2027-08-29", Monthly);
//         check_entry!("2027-09-05", Monthly);
//         check_entry!("2027-09-12", Monthly);
//         check_entry!("2027-09-19", Monthly);
//         check_entry!("2027-09-26", Monthly);
//         check_entry!("2027-10-03", Monthly);
//         check_entry!("2027-10-10", Monthly);
//         check_entry!("2027-10-17", Monthly);
//         check_entry!("2027-10-24", Monthly);
//         check_entry!("2027-10-31", Monthly);
//         check_entry!("2027-11-07", Monthly);
//         check_entry!("2027-11-14", Monthly);
//         check_entry!("2027-11-21", Monthly);
//         check_entry!("2027-11-28", Monthly);
//         check_entry!("2027-12-05", Monthly);
//         check_entry!("2027-12-12", Monthly);
//         check_entry!("2027-12-19", Monthly);
//         check_entry!("2027-12-26", Monthly);
//         check_entry!("2028-01-02", Monthly);
//         check_entry!("2028-01-09", Monthly);
//         check_entry!("2028-01-16", Monthly);
//         check_entry!("2028-01-23", Monthly);
//         check_entry!("2028-01-30", Monthly);
//         check_entry!("2028-02-06", Monthly);
//         check_entry!("2028-02-13", Monthly);
//         check_entry!("2028-02-20", Monthly);
//         check_entry!("2028-02-27", Monthly);
//         check_entry!("2028-03-05", Monthly);
//         check_entry!("2028-03-12", Monthly);
//         check_entry!("2028-03-19", Monthly);
//         check_entry!("2028-03-26", Monthly);
//         check_entry!("2028-04-02", Monthly);
//         check_entry!("2028-04-09", Monthly);
//         check_entry!("2028-04-16", Monthly);
//         check_entry!("2028-04-23", Monthly);
//         check_entry!("2028-04-30", Monthly);
//         check_entry!("2028-05-07", Monthly);
//         check_entry!("2028-05-14", Monthly);
//         check_entry!("2028-05-21", Monthly);
//         check_entry!("2028-05-28", Monthly);
//         check_entry!("2028-06-04", Monthly);
//         check_entry!("2028-06-11", Monthly);
//         check_entry!("2028-06-18", Monthly);
//         check_entry!("2028-06-25", Monthly);
//         check_entry!("2028-07-02", Monthly);
//         check_entry!("2028-07-09", Monthly);
//         check_entry!("2028-07-16", Monthly);
//         check_entry!("2028-07-23", Monthly);
//         check_entry!("2028-07-30", Monthly);
//         check_entry!("2028-08-06", Monthly);
//         check_entry!("2028-08-13", Monthly);
//         check_entry!("2028-08-20", Monthly);
//         check_entry!("2028-08-27", Monthly);
//         check_entry!("2028-09-03", Monthly);
//         check_entry!("2028-09-10", Monthly);
//         check_entry!("2028-09-17", Monthly);
//         check_entry!("2028-09-24", Monthly);
//         check_entry!("2028-10-01", Monthly);
//         check_entry!("2028-10-08", Monthly);
//         check_entry!("2028-10-15", Monthly);
//         check_entry!("2028-10-22", Monthly);
//         check_entry!("2028-10-29", Monthly);
//         check_entry!("2028-11-05", Monthly);
//         check_entry!("2028-11-12", Monthly);
//         check_entry!("2028-11-19", Monthly);
//         check_entry!("2028-11-26", Monthly);
//         check_entry!("2028-12-03", Monthly);
//         check_entry!("2028-12-10", Monthly);
//         check_entry!("2028-12-17", Monthly);
//         check_entry!("2028-12-24", Monthly);
//         check_entry!("2028-12-31", Monthly);
//         check_entry!("2029-01-07", Monthly);
//         check_entry!("2029-01-14", Monthly);
//         check_entry!("2029-01-21", Monthly);
//         check_entry!("2029-01-28", Monthly);
//         check_entry!("2029-02-04", Monthly);
//         check_entry!("2029-02-11", Monthly);
//         check_entry!("2029-02-18", Monthly);
//         check_entry!("2029-02-25", Monthly);
//         check_entry!("2029-03-04", Monthly);
//         check_entry!("2029-03-11", Monthly);
//         check_entry!("2029-03-18", Monthly);
//         check_entry!("2029-03-25", Monthly);
//         check_entry!("2029-04-01", Monthly);
//         check_entry!("2029-04-08", Monthly);
//         check_entry!("2029-04-15", Monthly);
//         check_entry!("2029-04-22", Monthly);
//         check_entry!("2029-04-29", Monthly);
//         check_entry!("2029-05-06", Monthly);
//         check_entry!("2029-05-13", Monthly);
//         check_entry!("2029-05-20", Monthly);
//         check_entry!("2029-05-27", Monthly);
//         check_entry!("2029-06-03", Monthly);
//         check_entry!("2029-06-10", Monthly);
//         check_entry!("2029-06-17", Monthly);
//         check_entry!("2029-06-24", Monthly);
//         check_entry!("2029-07-01", Monthly);
//         check_entry!("2029-07-08", Monthly);
//         check_entry!("2029-07-15", Monthly);
//         check_entry!("2029-07-22", Monthly);
//         check_entry!("2029-07-29", Monthly);
//         check_entry!("2029-08-05", Monthly);
//         check_entry!("2029-08-12", Monthly);
//         check_entry!("2029-08-19", Monthly);
//         check_entry!("2029-08-26", Monthly);
//         check_entry!("2029-09-02", Monthly);
//         check_entry!("2029-09-09", Monthly);
//         check_entry!("2029-09-16", Monthly);
//         check_entry!("2029-09-23", Monthly);
//         check_entry!("2029-09-30", Monthly);
//         check_entry!("2029-10-07", Monthly);
//         check_entry!("2029-10-14", Monthly);
//         check_entry!("2029-10-21", Monthly);
//         check_entry!("2029-10-28", Monthly);
//         check_entry!("2029-11-04", Monthly);
//         check_entry!("2029-11-11", Monthly);
//         check_entry!("2029-11-18", Monthly);
//         check_entry!("2029-11-25", Monthly);
//         check_entry!("2029-12-02", Monthly);
//         check_entry!("2029-12-09", Monthly);
//         check_entry!("2029-12-16", Monthly);
//         check_entry!("2029-12-23", Monthly);
//         check_entry!("2029-12-30", Monthly);
//         check_entry!("2030-01-06", Monthly);
//         check_entry!("2030-01-13", Monthly);
//         check_entry!("2030-01-20", Monthly);
//         check_entry!("2030-01-27", Monthly);
//         check_entry!("2030-02-03", Monthly);
//         check_entry!("2030-02-10", Monthly);
//         check_entry!("2030-02-17", Monthly);
//         check_entry!("2030-02-24", Monthly);
//         check_entry!("2030-03-03", Monthly);
//         check_entry!("2030-03-10", Monthly);
//         check_entry!("2030-03-17", Monthly);
//         check_entry!("2030-03-24", Monthly);
//         check_entry!("2030-03-31", Monthly);
//         check_entry!("2030-04-07", Monthly);
//         check_entry!("2030-04-14", Monthly);
//         check_entry!("2030-04-21", Monthly);
//         check_entry!("2030-04-28", Monthly);
//         check_entry!("2030-05-05", Monthly);
//         check_entry!("2030-05-12", Monthly);
//         check_entry!("2030-05-19", Monthly);
//         check_entry!("2030-05-26", Monthly);
//         check_entry!("2030-06-02", Monthly);
//         check_entry!("2030-06-09", Monthly);
//         check_entry!("2030-06-16", Monthly);
//         check_entry!("2030-06-23", Monthly);
//         check_entry!("2030-06-30", Monthly);
//         check_entry!("2030-07-07", Monthly);
//         check_entry!("2030-07-14", Monthly);
//         check_entry!("2030-07-21", Monthly);
//         check_entry!("2030-07-28", Monthly);
//         check_entry!("2030-08-04", Monthly);
//         check_entry!("2030-08-11", Monthly);
//         check_entry!("2030-08-18", Monthly);
//         check_entry!("2030-08-25", Monthly);
//         check_entry!("2030-09-01", Monthly);
//         check_entry!("2030-09-08", Monthly);
//         check_entry!("2030-09-15", Monthly);
//         check_entry!("2030-09-22", Monthly);
//         check_entry!("2030-09-29", Monthly);
//         check_entry!("2030-10-06", Monthly);
//         check_entry!("2030-10-13", Monthly);
//         check_entry!("2030-10-20", Monthly);
//         check_entry!("2030-10-27", Monthly);
//         check_entry!("2030-11-03", Monthly);
//         check_entry!("2030-11-10", Monthly);
//         check_entry!("2030-11-17", Monthly);
//         check_entry!("2030-11-24", Monthly);
//         check_entry!("2030-12-01", Monthly);
//         check_entry!("2030-12-08", Monthly);
//         check_entry!("2030-12-15", Monthly);
//         check_entry!("2030-12-22", Monthly);
//         check_entry!("2030-12-29", Monthly);
//         check_entry!("2031-01-05", Monthly);
//         check_entry!("2031-01-12", Monthly);
//         check_entry!("2031-01-19", Monthly);
//         check_entry!("2031-01-26", Monthly);
//         check_entry!("2031-02-02", Monthly);
//         check_entry!("2031-02-09", Monthly);
//         check_entry!("2031-02-16", Monthly);
//         check_entry!("2031-02-23", Monthly);
//         check_entry!("2031-03-02", Monthly);
//         check_entry!("2031-03-09", Monthly);
//         check_entry!("2031-03-16", Monthly);
//         check_entry!("2031-03-23", Monthly);
//         check_entry!("2031-03-30", Monthly);
//         check_entry!("2031-04-06", Monthly);
//         check_entry!("2031-04-13", Monthly);
//         check_entry!("2031-04-20", Monthly);
//         check_entry!("2031-04-27", Monthly);
//         check_entry!("2031-05-04", Monthly);
//         check_entry!("2031-05-11", Monthly);
//         check_entry!("2031-05-18", Monthly);
//         check_entry!("2031-05-25", Monthly);
//         check_entry!("2031-06-01", Monthly);
//         check_entry!("2031-06-08", Monthly);
//         check_entry!("2031-06-15", Monthly);
//         check_entry!("2031-06-22", Monthly);
//         check_entry!("2031-06-29", Monthly);
//         check_entry!("2031-07-06", Monthly);
//         check_entry!("2031-07-13", Monthly);
//         check_entry!("2031-07-20", Monthly);
//         check_entry!("2031-07-27", Monthly);
//         check_entry!("2031-08-03", Monthly);
//         check_entry!("2031-08-10", Monthly);
//         check_entry!("2031-08-17", Monthly);
//         check_entry!("2031-08-24", Monthly);
//         check_entry!("2031-08-31", Monthly);
//         check_entry!("2031-09-07", Monthly);
//         check_entry!("2031-09-14", Monthly);
//         check_entry!("2031-09-21", Monthly);
//         check_entry!("2031-09-28", Monthly);
//         check_entry!("2031-10-05", Monthly);
//         check_entry!("2031-10-12", Monthly);
//         check_entry!("2031-10-19", Monthly);
//         check_entry!("2031-10-26", Monthly);
//         check_entry!("2031-11-02", Monthly);
//         check_entry!("2031-11-09", Monthly);
//         check_entry!("2031-11-16", Monthly);
//         check_entry!("2031-11-23", Monthly);
//         check_entry!("2031-11-30", Monthly);
//         check_entry!("2031-12-07", Monthly);
//         check_entry!("2031-12-14", Monthly);
//         check_entry!("2031-12-21", Monthly);
//         check_entry!("2031-12-28", Monthly);
//         check_entry!("2032-01-04", Monthly);
//         check_entry!("2032-01-11", Monthly);
//         check_entry!("2032-01-18", Monthly);
//         check_entry!("2032-01-25", Monthly);
//         check_entry!("2032-02-01", Monthly);
//         check_entry!("2032-02-08", Monthly);
//         check_entry!("2032-02-15", Monthly);
//         check_entry!("2032-02-22", Monthly);
//         check_entry!("2032-02-29", Monthly);
//         check_entry!("2032-03-07", Monthly);
//         check_entry!("2032-03-14", Monthly);
//         check_entry!("2032-03-21", Monthly);
//         check_entry!("2032-03-28", Monthly);
//         check_entry!("2032-04-04", Monthly);
//         check_entry!("2032-04-11", Monthly);
//         check_entry!("2032-04-18", Monthly);
//         check_entry!("2032-04-25", Monthly);
//         check_entry!("2032-05-02", Monthly);
//         check_entry!("2032-05-09", Monthly);
//         check_entry!("2032-05-16", Monthly);
//         check_entry!("2032-05-23", Monthly);
//         check_entry!("2032-05-30", Monthly);
//         check_entry!("2032-06-06", Monthly);
//         check_entry!("2032-06-13", Monthly);
//         check_entry!("2032-06-20", Monthly);
//         check_entry!("2032-06-27", Monthly);
//         check_entry!("2032-07-04", Monthly);
//         check_entry!("2032-07-11", Monthly);
//         check_entry!("2032-07-18", Monthly);
//         check_entry!("2032-07-25", Monthly);
//         check_entry!("2032-08-01", Monthly);
//         check_entry!("2032-08-08", Monthly);
//
//         check_entry!("2032-08-15", Done);
//         check_entry!("2032-08-22", Done);
//         check_entry!("2032-08-29", Done);
//         check_entry!("2032-09-05", Done);
//         check_entry!("2032-09-12", Done);
//         check_entry!("2032-09-19", Done);
//         check_entry!("2032-09-26", Done);
//         check_entry!("2032-10-03", Done);
//         check_entry!("2032-10-10", Done);
//         check_entry!("2032-10-17", Done);
//         check_entry!("2032-10-24", Done);
//         check_entry!("2032-10-31", Done);
//         check_entry!("2032-11-07", Done);
//         check_entry!("2032-11-14", Done);
//         check_entry!("2032-11-21", Done);
//         check_entry!("2032-11-28", Done);
//         check_entry!("2032-12-05", Done);
//         check_entry!("2032-12-12", Done);
//         check_entry!("2032-12-19", Done);
//         check_entry!("2032-12-26", Done);
//         check_entry!("2033-01-02", Done);
//         check_entry!("2033-01-09", Done);
//         check_entry!("2033-01-16", Done);
//         check_entry!("2033-01-23", Done);
//         check_entry!("2033-01-30", Done);
//         check_entry!("2033-02-06", Done);
//
//         Ok(())
//     }
//
//     #[test]
//     fn john() -> AnyResult<()> {
//         let date = "2025-07-06";
//         let references = vec![
//             VerseEntry::new("2025-07-06", "John 1:1")?,
//             VerseEntry::new("2025-07-13", "John 1:2")?,
//             VerseEntry::new("2025-07-20", "John 1:3")?,
//             VerseEntry::new("2025-07-27", "John 1:4")?,
//             VerseEntry::new("2025-08-03", "John 1:5")?,
//             VerseEntry::new("2025-08-10", "John 1:6")?,
//             VerseEntry::new("2025-08-17", "John 1:7")?,
//             VerseEntry::new("2025-08-24", "John 1:8")?,
//             VerseEntry::new("2025-08-31", "John 1:9")?,
//             VerseEntry::new("2025-09-07", "John 1:10")?,
//             VerseEntry::new("2025-09-14", "John 1:11")?,
//             VerseEntry::new("2025-09-21", "John 1:12")?,
//             VerseEntry::new("2025-09-28", "John 1:13")?,
//             VerseEntry::new("2025-10-05", "John 1:14")?,
//             VerseEntry::new("2025-10-12", "John 1:15")?,
//             VerseEntry::new("2025-10-19", "John 1:16")?,
//             VerseEntry::new("2025-10-26", "John 1:17")?,
//             VerseEntry::new("2025-11-02", "John 1:18")?,
//             VerseEntry::new("2025-11-09", "John 1:19")?,
//             VerseEntry::new("2025-11-16", "John 1:20")?,
//             VerseEntry::new("2025-11-23", "John 1:21")?,
//             VerseEntry::new("2025-11-30", "John 1:22")?,
//             VerseEntry::new("2025-12-07", "John 1:23")?,
//             VerseEntry::new("2025-12-14", "John 1:24")?,
//             VerseEntry::new("2025-12-21", "John 1:25")?,
//             VerseEntry::new("2025-12-28", "John 1:26")?,
//             VerseEntry::new("2026-01-04", "John 1:27")?,
//             VerseEntry::new("2026-01-11", "John 1:28")?,
//             VerseEntry::new("2026-01-18", "John 1:29")?,
//             VerseEntry::new("2026-01-25", "John 1:30")?,
//             VerseEntry::new("2026-02-01", "John 1:31")?,
//             VerseEntry::new("2026-02-08", "John 1:32")?,
//             VerseEntry::new("2026-02-15", "John 1:33")?,
//             VerseEntry::new("2026-02-22", "John 1:34")?,
//             VerseEntry::new("2026-03-01", "John 1:35")?,
//             VerseEntry::new("2026-03-08", "John 1:36")?,
//             VerseEntry::new("2026-03-15", "John 1:37")?,
//             VerseEntry::new("2026-03-22", "John 1:38")?,
//             VerseEntry::new("2026-03-29", "John 1:39")?,
//             VerseEntry::new("2026-04-05", "John 1:40")?,
//             VerseEntry::new("2026-04-12", "John 1:41")?,
//             VerseEntry::new("2026-04-19", "John 1:42")?,
//             VerseEntry::new("2026-04-26", "John 1:43")?,
//             VerseEntry::new("2026-05-03", "John 1:44")?,
//             VerseEntry::new("2026-05-10", "John 1:45")?,
//             VerseEntry::new("2026-05-17", "John 1:46")?,
//             VerseEntry::new("2026-05-24", "John 1:47")?,
//             VerseEntry::new("2026-05-31", "John 1:48")?,
//             VerseEntry::new("2026-06-07", "John 1:49")?,
//             VerseEntry::new("2026-06-14", "John 1:50")?,
//             VerseEntry::new("2026-06-21", "John 1:51")?,
//             VerseEntry::new("2026-06-28", "John 2:1")?,
//             VerseEntry::new("2026-07-05", "John 2:2")?,
//             VerseEntry::new("2026-07-12", "John 2:3")?,
//             VerseEntry::new("2026-07-19", "John 2:4")?,
//             VerseEntry::new("2026-07-26", "John 2:5")?,
//             VerseEntry::new("2026-08-02", "John 2:6")?,
//             VerseEntry::new("2026-08-09", "John 2:7")?,
//             VerseEntry::new("2026-08-16", "John 2:8")?,
//             VerseEntry::new("2026-08-23", "John 2:9")?,
//             VerseEntry::new("2026-08-30", "John 2:10")?,
//             VerseEntry::new("2026-09-06", "John 2:11")?,
//             VerseEntry::new("2026-09-13", "John 2:12")?,
//             VerseEntry::new("2026-09-20", "John 2:13")?,
//             VerseEntry::new("2026-09-27", "John 2:14")?,
//             VerseEntry::new("2026-10-04", "John 2:15")?,
//             VerseEntry::new("2026-10-11", "John 2:16")?,
//             VerseEntry::new("2026-10-18", "John 2:17")?,
//             VerseEntry::new("2026-10-25", "John 2:18")?,
//             VerseEntry::new("2026-11-01", "John 2:19")?,
//             VerseEntry::new("2026-11-08", "John 2:20")?,
//             VerseEntry::new("2026-11-15", "John 2:21")?,
//             VerseEntry::new("2026-11-22", "John 2:22")?,
//             VerseEntry::new("2026-11-29", "John 2:23")?,
//             VerseEntry::new("2026-12-06", "John 2:24")?,
//             VerseEntry::new("2026-12-13", "John 2:25")?,
//             VerseEntry::new("2026-12-20", "John 3:1")?,
//             VerseEntry::new("2026-12-27", "John 3:2")?,
//             VerseEntry::new("2027-01-03", "John 3:3")?,
//             VerseEntry::new("2027-01-10", "John 3:4")?,
//             VerseEntry::new("2027-01-17", "John 3:5")?,
//             VerseEntry::new("2027-01-24", "John 3:6")?,
//             VerseEntry::new("2027-01-31", "John 3:7")?,
//             VerseEntry::new("2027-02-07", "John 3:8")?,
//             VerseEntry::new("2027-02-14", "John 3:9")?,
//             VerseEntry::new("2027-02-21", "John 3:10")?,
//             VerseEntry::new("2027-02-28", "John 3:11")?,
//             VerseEntry::new("2027-03-07", "John 3:12")?,
//             VerseEntry::new("2027-03-14", "John 3:13")?,
//             VerseEntry::new("2027-03-21", "John 3:14")?,
//             VerseEntry::new("2027-03-28", "John 3:15")?,
//             VerseEntry::new("2027-04-04", "John 3:16")?,
//             VerseEntry::new("2027-04-11", "John 3:17")?,
//             VerseEntry::new("2027-04-18", "John 3:18")?,
//             VerseEntry::new("2027-04-25", "John 3:19")?,
//             VerseEntry::new("2027-05-02", "John 3:20")?,
//             VerseEntry::new("2027-05-09", "John 3:21")?,
//             VerseEntry::new("2027-05-16", "John 3:22")?,
//             VerseEntry::new("2027-05-23", "John 3:23")?,
//             VerseEntry::new("2027-05-30", "John 3:24")?,
//             VerseEntry::new("2027-06-06", "John 3:25")?,
//             VerseEntry::new("2027-06-13", "John 3:26")?,
//             VerseEntry::new("2027-06-20", "John 3:27")?,
//             VerseEntry::new("2027-06-27", "John 3:28")?,
//             VerseEntry::new("2027-07-04", "John 3:29")?,
//             VerseEntry::new("2027-07-11", "John 3:30")?,
//             VerseEntry::new("2027-07-18", "John 3:31")?,
//             VerseEntry::new("2027-07-25", "John 3:32")?,
//             VerseEntry::new("2027-08-01", "John 3:33")?,
//             VerseEntry::new("2027-08-08", "John 3:34")?,
//             VerseEntry::new("2027-08-15", "John 3:35")?,
//             VerseEntry::new("2027-08-22", "John 3:36")?,
//             VerseEntry::new("2027-08-29", "John 4:1")?,
//             VerseEntry::new("2027-09-05", "John 4:2")?,
//             VerseEntry::new("2027-09-12", "John 4:3")?,
//             VerseEntry::new("2027-09-19", "John 4:4")?,
//             VerseEntry::new("2027-09-26", "John 4:5")?,
//             VerseEntry::new("2027-10-03", "John 4:6")?,
//             VerseEntry::new("2027-10-10", "John 4:7")?,
//             VerseEntry::new("2027-10-17", "John 4:8")?,
//             VerseEntry::new("2027-10-24", "John 4:9")?,
//             VerseEntry::new("2027-10-31", "John 4:10")?,
//             VerseEntry::new("2027-11-07", "John 4:11")?,
//             VerseEntry::new("2027-11-14", "John 4:12")?,
//             VerseEntry::new("2027-11-21", "John 4:13")?,
//             VerseEntry::new("2027-11-28", "John 4:14")?,
//             VerseEntry::new("2027-12-05", "John 4:15")?,
//             VerseEntry::new("2027-12-12", "John 4:16")?,
//             VerseEntry::new("2027-12-19", "John 4:17")?,
//             VerseEntry::new("2027-12-26", "John 4:18")?,
//             VerseEntry::new("2028-01-02", "John 4:19")?,
//             VerseEntry::new("2028-01-09", "John 4:20")?,
//             VerseEntry::new("2028-01-16", "John 4:21")?,
//             VerseEntry::new("2028-01-23", "John 4:22")?,
//             VerseEntry::new("2028-01-30", "John 4:23")?,
//             VerseEntry::new("2028-02-06", "John 4:24")?,
//             VerseEntry::new("2028-02-13", "John 4:25")?,
//             VerseEntry::new("2028-02-20", "John 4:26")?,
//             VerseEntry::new("2028-02-27", "John 4:27")?,
//             VerseEntry::new("2028-03-05", "John 4:28")?,
//             VerseEntry::new("2028-03-12", "John 4:29")?,
//             VerseEntry::new("2028-03-19", "John 4:30")?,
//             VerseEntry::new("2028-03-26", "John 4:31")?,
//             VerseEntry::new("2028-04-02", "John 4:32")?,
//             VerseEntry::new("2028-04-09", "John 4:33")?,
//             VerseEntry::new("2028-04-16", "John 4:34")?,
//             VerseEntry::new("2028-04-23", "John 4:35")?,
//             VerseEntry::new("2028-04-30", "John 4:36")?,
//             VerseEntry::new("2028-05-07", "John 4:37")?,
//             VerseEntry::new("2028-05-14", "John 4:38")?,
//             VerseEntry::new("2028-05-21", "John 4:39")?,
//             VerseEntry::new("2028-05-28", "John 4:40")?,
//             VerseEntry::new("2028-06-04", "John 4:41")?,
//             VerseEntry::new("2028-06-11", "John 4:42")?,
//             VerseEntry::new("2028-06-18", "John 4:43")?,
//             VerseEntry::new("2028-06-25", "John 4:44")?,
//             VerseEntry::new("2028-07-02", "John 4:45")?,
//             VerseEntry::new("2028-07-09", "John 4:46")?,
//             VerseEntry::new("2028-07-16", "John 4:47")?,
//             VerseEntry::new("2028-07-23", "John 4:48")?,
//             VerseEntry::new("2028-07-30", "John 4:49")?,
//             VerseEntry::new("2028-08-06", "John 4:50")?,
//             VerseEntry::new("2028-08-13", "John 4:51")?,
//             VerseEntry::new("2028-08-20", "John 4:52")?,
//             VerseEntry::new("2028-08-27", "John 4:53")?,
//             VerseEntry::new("2028-09-03", "John 4:54")?,
//             VerseEntry::new("2028-09-10", "John 5:1")?,
//             VerseEntry::new("2028-09-17", "John 5:2")?,
//             VerseEntry::new("2028-09-24", "John 5:3")?,
//             VerseEntry::new("2028-10-01", "John 5:4")?,
//             VerseEntry::new("2028-10-08", "John 5:5")?,
//             VerseEntry::new("2028-10-15", "John 5:6")?,
//             VerseEntry::new("2028-10-22", "John 5:7")?,
//             VerseEntry::new("2028-10-29", "John 5:8")?,
//             VerseEntry::new("2028-11-05", "John 5:9")?,
//             VerseEntry::new("2028-11-12", "John 5:10")?,
//             VerseEntry::new("2028-11-19", "John 5:11")?,
//             VerseEntry::new("2028-11-26", "John 5:12")?,
//             VerseEntry::new("2028-12-03", "John 5:13")?,
//             VerseEntry::new("2028-12-10", "John 5:14")?,
//             VerseEntry::new("2028-12-17", "John 5:15")?,
//             VerseEntry::new("2028-12-24", "John 5:16")?,
//             VerseEntry::new("2028-12-31", "John 5:17")?,
//             VerseEntry::new("2029-01-07", "John 5:18")?,
//             VerseEntry::new("2029-01-14", "John 5:19")?,
//             VerseEntry::new("2029-01-21", "John 5:20")?,
//             VerseEntry::new("2029-01-28", "John 5:21")?,
//             VerseEntry::new("2029-02-04", "John 5:22")?,
//             VerseEntry::new("2029-02-11", "John 5:23")?,
//             VerseEntry::new("2029-02-18", "John 5:24")?,
//             VerseEntry::new("2029-02-25", "John 5:25")?,
//             VerseEntry::new("2029-03-04", "John 5:26")?,
//             VerseEntry::new("2029-03-11", "John 5:27")?,
//             VerseEntry::new("2029-03-18", "John 5:28")?,
//             VerseEntry::new("2029-03-25", "John 5:29")?,
//             VerseEntry::new("2029-04-01", "John 5:30")?,
//             VerseEntry::new("2029-04-08", "John 5:31")?,
//             VerseEntry::new("2029-04-15", "John 5:32")?,
//             VerseEntry::new("2029-04-22", "John 5:33")?,
//             VerseEntry::new("2029-04-29", "John 5:34")?,
//             VerseEntry::new("2029-05-06", "John 5:35")?,
//             VerseEntry::new("2029-05-13", "John 5:36")?,
//             VerseEntry::new("2029-05-20", "John 5:37")?,
//             VerseEntry::new("2029-05-27", "John 5:38")?,
//             VerseEntry::new("2029-06-03", "John 5:39")?,
//             VerseEntry::new("2029-06-10", "John 5:40")?,
//             VerseEntry::new("2029-06-17", "John 5:41")?,
//             VerseEntry::new("2029-06-24", "John 5:42")?,
//             VerseEntry::new("2029-07-01", "John 5:43")?,
//             VerseEntry::new("2029-07-08", "John 5:44")?,
//             VerseEntry::new("2029-07-15", "John 5:45")?,
//             VerseEntry::new("2029-07-22", "John 5:46")?,
//             VerseEntry::new("2029-07-29", "John 5:47")?,
//             VerseEntry::new("2029-08-05", "John 6:1")?,
//             VerseEntry::new("2029-08-12", "John 6:2")?,
//             VerseEntry::new("2029-08-19", "John 6:3")?,
//             VerseEntry::new("2029-08-26", "John 6:4")?,
//             VerseEntry::new("2029-09-02", "John 6:5")?,
//             VerseEntry::new("2029-09-09", "John 6:6")?,
//             VerseEntry::new("2029-09-16", "John 6:7")?,
//             VerseEntry::new("2029-09-23", "John 6:8")?,
//             VerseEntry::new("2029-09-30", "John 6:9")?,
//             VerseEntry::new("2029-10-07", "John 6:10")?,
//             VerseEntry::new("2029-10-14", "John 6:11")?,
//             VerseEntry::new("2029-10-21", "John 6:12")?,
//             VerseEntry::new("2029-10-28", "John 6:13")?,
//             VerseEntry::new("2029-11-04", "John 6:14")?,
//             VerseEntry::new("2029-11-11", "John 6:15")?,
//             VerseEntry::new("2029-11-18", "John 6:16")?,
//             VerseEntry::new("2029-11-25", "John 6:17")?,
//             VerseEntry::new("2029-12-02", "John 6:18")?,
//             VerseEntry::new("2029-12-09", "John 6:19")?,
//             VerseEntry::new("2029-12-16", "John 6:20")?,
//             VerseEntry::new("2029-12-23", "John 6:21")?,
//             VerseEntry::new("2029-12-30", "John 6:22")?,
//             VerseEntry::new("2030-01-06", "John 6:23")?,
//             VerseEntry::new("2030-01-13", "John 6:24")?,
//             VerseEntry::new("2030-01-20", "John 6:25")?,
//             VerseEntry::new("2030-01-27", "John 6:26")?,
//             VerseEntry::new("2030-02-03", "John 6:27")?,
//             VerseEntry::new("2030-02-10", "John 6:28")?,
//             VerseEntry::new("2030-02-17", "John 6:29")?,
//             VerseEntry::new("2030-02-24", "John 6:30")?,
//             VerseEntry::new("2030-03-03", "John 6:31")?,
//             VerseEntry::new("2030-03-10", "John 6:32")?,
//             VerseEntry::new("2030-03-17", "John 6:33")?,
//             VerseEntry::new("2030-03-24", "John 6:34")?,
//             VerseEntry::new("2030-03-31", "John 6:35")?,
//             VerseEntry::new("2030-04-07", "John 6:36")?,
//             VerseEntry::new("2030-04-14", "John 6:37")?,
//             VerseEntry::new("2030-04-21", "John 6:38")?,
//             VerseEntry::new("2030-04-28", "John 6:39")?,
//             VerseEntry::new("2030-05-05", "John 6:40")?,
//             VerseEntry::new("2030-05-12", "John 6:41")?,
//             VerseEntry::new("2030-05-19", "John 6:42")?,
//             VerseEntry::new("2030-05-26", "John 6:43")?,
//             VerseEntry::new("2030-06-02", "John 6:44")?,
//             VerseEntry::new("2030-06-09", "John 6:45")?,
//             VerseEntry::new("2030-06-16", "John 6:46")?,
//             VerseEntry::new("2030-06-23", "John 6:47")?,
//             VerseEntry::new("2030-06-30", "John 6:48")?,
//             VerseEntry::new("2030-07-07", "John 6:49")?,
//             VerseEntry::new("2030-07-14", "John 6:50")?,
//             VerseEntry::new("2030-07-21", "John 6:51")?,
//             VerseEntry::new("2030-07-28", "John 6:52")?,
//             VerseEntry::new("2030-08-04", "John 6:53")?,
//             VerseEntry::new("2030-08-11", "John 6:54")?,
//             VerseEntry::new("2030-08-18", "John 6:55")?,
//             VerseEntry::new("2030-08-25", "John 6:56")?,
//             VerseEntry::new("2030-09-01", "John 6:57")?,
//             VerseEntry::new("2030-09-08", "John 6:58")?,
//             VerseEntry::new("2030-09-15", "John 6:59")?,
//             VerseEntry::new("2030-09-22", "John 6:60")?,
//             VerseEntry::new("2030-09-29", "John 6:61")?,
//             VerseEntry::new("2030-10-06", "John 6:62")?,
//             VerseEntry::new("2030-10-13", "John 6:63")?,
//             VerseEntry::new("2030-10-20", "John 6:64")?,
//             VerseEntry::new("2030-10-27", "John 6:65")?,
//             VerseEntry::new("2030-11-03", "John 6:66")?,
//             VerseEntry::new("2030-11-10", "John 6:67")?,
//             VerseEntry::new("2030-11-17", "John 6:68")?,
//             VerseEntry::new("2030-11-24", "John 6:69")?,
//             VerseEntry::new("2030-12-01", "John 6:70")?,
//             VerseEntry::new("2030-12-08", "John 6:71")?,
//             VerseEntry::new("2030-12-15", "John 7:1")?,
//             VerseEntry::new("2030-12-22", "John 7:2")?,
//             VerseEntry::new("2030-12-29", "John 7:3")?,
//             VerseEntry::new("2031-01-05", "John 7:4")?,
//             VerseEntry::new("2031-01-12", "John 7:5")?,
//             VerseEntry::new("2031-01-19", "John 7:6")?,
//             VerseEntry::new("2031-01-26", "John 7:7")?,
//             VerseEntry::new("2031-02-02", "John 7:8")?,
//             VerseEntry::new("2031-02-09", "John 7:9")?,
//             VerseEntry::new("2031-02-16", "John 7:10")?,
//             VerseEntry::new("2031-02-23", "John 7:11")?,
//             VerseEntry::new("2031-03-02", "John 7:12")?,
//             VerseEntry::new("2031-03-09", "John 7:13")?,
//             VerseEntry::new("2031-03-16", "John 7:14")?,
//             VerseEntry::new("2031-03-23", "John 7:15")?,
//             VerseEntry::new("2031-03-30", "John 7:16")?,
//             VerseEntry::new("2031-04-06", "John 7:17")?,
//             VerseEntry::new("2031-04-13", "John 7:18")?,
//             VerseEntry::new("2031-04-20", "John 7:19")?,
//             VerseEntry::new("2031-04-27", "John 7:20")?,
//             VerseEntry::new("2031-05-04", "John 7:21")?,
//             VerseEntry::new("2031-05-11", "John 7:22")?,
//             VerseEntry::new("2031-05-18", "John 7:23")?,
//             VerseEntry::new("2031-05-25", "John 7:24")?,
//             VerseEntry::new("2031-06-01", "John 7:25")?,
//             VerseEntry::new("2031-06-08", "John 7:26")?,
//             VerseEntry::new("2031-06-15", "John 7:27")?,
//             VerseEntry::new("2031-06-22", "John 7:28")?,
//             VerseEntry::new("2031-06-29", "John 7:29")?,
//             VerseEntry::new("2031-07-06", "John 7:30")?,
//             VerseEntry::new("2031-07-13", "John 7:31")?,
//             VerseEntry::new("2031-07-20", "John 7:32")?,
//             VerseEntry::new("2031-07-27", "John 7:33")?,
//             VerseEntry::new("2031-08-03", "John 7:34")?,
//             VerseEntry::new("2031-08-10", "John 7:35")?,
//             VerseEntry::new("2031-08-17", "John 7:36")?,
//             VerseEntry::new("2031-08-24", "John 7:37")?,
//             VerseEntry::new("2031-08-31", "John 7:38")?,
//             VerseEntry::new("2031-09-07", "John 7:39")?,
//             VerseEntry::new("2031-09-14", "John 7:40")?,
//             VerseEntry::new("2031-09-21", "John 7:41")?,
//             VerseEntry::new("2031-09-28", "John 7:42")?,
//             VerseEntry::new("2031-10-05", "John 7:43")?,
//             VerseEntry::new("2031-10-12", "John 7:44")?,
//             VerseEntry::new("2031-10-19", "John 7:45")?,
//             VerseEntry::new("2031-10-26", "John 7:46")?,
//             VerseEntry::new("2031-11-02", "John 7:47")?,
//             VerseEntry::new("2031-11-09", "John 7:48")?,
//             VerseEntry::new("2031-11-16", "John 7:49")?,
//             VerseEntry::new("2031-11-23", "John 7:50")?,
//             VerseEntry::new("2031-11-30", "John 7:51")?,
//             VerseEntry::new("2031-12-07", "John 7:52")?,
//             VerseEntry::new("2031-12-14", "John 7:53")?,
//             VerseEntry::new("2031-12-21", "John 8:1")?,
//             VerseEntry::new("2031-12-28", "John 8:2")?,
//             VerseEntry::new("2032-01-04", "John 8:3")?,
//             VerseEntry::new("2032-01-11", "John 8:4")?,
//             VerseEntry::new("2032-01-18", "John 8:5")?,
//             VerseEntry::new("2032-01-25", "John 8:6")?,
//             VerseEntry::new("2032-02-01", "John 8:7")?,
//             VerseEntry::new("2032-02-08", "John 8:8")?,
//             VerseEntry::new("2032-02-15", "John 8:9")?,
//             VerseEntry::new("2032-02-22", "John 8:10")?,
//             VerseEntry::new("2032-02-29", "John 8:11")?,
//             VerseEntry::new("2032-03-07", "John 8:12")?,
//             VerseEntry::new("2032-03-14", "John 8:13")?,
//             VerseEntry::new("2032-03-21", "John 8:14")?,
//             VerseEntry::new("2032-03-28", "John 8:15")?,
//             VerseEntry::new("2032-04-04", "John 8:16")?,
//             VerseEntry::new("2032-04-11", "John 8:17")?,
//             VerseEntry::new("2032-04-18", "John 8:18")?,
//             VerseEntry::new("2032-04-25", "John 8:19")?,
//             VerseEntry::new("2032-05-02", "John 8:20")?,
//             VerseEntry::new("2032-05-09", "John 8:21")?,
//             VerseEntry::new("2032-05-16", "John 8:22")?,
//             VerseEntry::new("2032-05-23", "John 8:23")?,
//             VerseEntry::new("2032-05-30", "John 8:24")?,
//             VerseEntry::new("2032-06-06", "John 8:25")?,
//             VerseEntry::new("2032-06-13", "John 8:26")?,
//             VerseEntry::new("2032-06-20", "John 8:27")?,
//             VerseEntry::new("2032-06-27", "John 8:28")?,
//             VerseEntry::new("2032-07-04", "John 8:29")?,
//             VerseEntry::new("2032-07-11", "John 8:30")?,
//             VerseEntry::new("2032-07-18", "John 8:31")?,
//             VerseEntry::new("2032-07-25", "John 8:32")?,
//             VerseEntry::new("2032-08-01", "John 8:33")?,
//             VerseEntry::new("2032-08-08", "John 8:34")?,
//             VerseEntry::new("2032-08-15", "John 8:35")?,
//             VerseEntry::new("2032-08-22", "John 8:36")?,
//             VerseEntry::new("2032-08-29", "John 8:37")?,
//             VerseEntry::new("2032-09-05", "John 8:38")?,
//             VerseEntry::new("2032-09-12", "John 8:39")?,
//             VerseEntry::new("2032-09-19", "John 8:40")?,
//             VerseEntry::new("2032-09-26", "John 8:41")?,
//             VerseEntry::new("2032-10-03", "John 8:42")?,
//             VerseEntry::new("2032-10-10", "John 8:43")?,
//             VerseEntry::new("2032-10-17", "John 8:44")?,
//             VerseEntry::new("2032-10-24", "John 8:45")?,
//             VerseEntry::new("2032-10-31", "John 8:46")?,
//             VerseEntry::new("2032-11-07", "John 8:47")?,
//             VerseEntry::new("2032-11-14", "John 8:48")?,
//             VerseEntry::new("2032-11-21", "John 8:49")?,
//             VerseEntry::new("2032-11-28", "John 8:50")?,
//             VerseEntry::new("2032-12-05", "John 8:51")?,
//             VerseEntry::new("2032-12-12", "John 8:52")?,
//             VerseEntry::new("2032-12-19", "John 8:53")?,
//             VerseEntry::new("2032-12-26", "John 8:54")?,
//             VerseEntry::new("2033-01-02", "John 8:55")?,
//             VerseEntry::new("2033-01-09", "John 8:56")?,
//             VerseEntry::new("2033-01-16", "John 8:57")?,
//             VerseEntry::new("2033-01-23", "John 8:58")?,
//             VerseEntry::new("2033-01-30", "John 8:59")?,
//             VerseEntry::new("2033-02-06", "John 9:1")?,
//         ];
//         let list = VerseList::new(date, references)?;
//         let verses = list.relative_verses();
//
//         // dbg!(VersesForAWeek::new(&verses, 1));
//         dbg!(VersesForAMonth::new(&verses));
//
//         Ok(())
//     }
// }
