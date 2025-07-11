#![allow(unused)]

pub type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

use std::borrow::Cow;

use chrono::{Datelike, NaiveDate};
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

    pub fn is_monthly(&self) -> bool {
        self.frequency() == Frequency::Monthly
    }

    pub fn will_be_monthly_this_month(&self, n: i64) -> bool {
        self.is_monthly() || self.with_offset(n).is_monthly()
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

#[derive(Clone, Debug, Default)]
pub struct VersesForADay<'a> {
    daily: Vec<Verse<'a>>,
    weekly: Vec<Verse<'a>>,
    monthly: Vec<Verse<'a>>,
}

impl<'a> VersesForADay<'a> {
    pub fn data(&self) -> String {
        let daily = self.daily.iter().map(|v| &v.reference).join("\n- ");
        let weekly = self.weekly.iter().map(|v| &v.reference).join("\n- ");
        let monthly = self.monthly.iter().map(|v| &v.reference).join("\n- ");
        vec![
            format!("### Daily: \n- {}", daily),
            format!("### Weekly: \n- {}", weekly),
            format!("### Monthly: \n- {}", monthly),
        ]
        .join("\n\n")
    }
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
            .filter(|verse| verse.with_offset(n).is_weekly())
            .cloned()
            .collect();

        let monthly: Vec<_> = verses
            .iter()
            // I actually want this to be "is monthly" or "will be monthly this month"
            // .filter(|verse| verse.is_monthly())
            .filter(|verse| verse.will_be_monthly_this_month(n))
            .cloned()
            .collect_vec();
        let bin = monthly.len() / 4;
        let monthly = monthly
            .into_iter()
            .skip(n as usize * bin)
            .take(bin)
            .collect_vec();

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
            .map(|n| VersesForAWeek::new(&verses, n))
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
                            "D: {} | W: {} | M: {}\n{}",
                            // "D: {} | W: {} | M: {}",
                            day.daily.len(),
                            day.weekly.len(),
                            day.monthly.len(),
                            day.monthly.iter().map(|v| &v.reference).join(" + "),
                        )
                    })
                    .join("\n")
            })
            .join("\n---\n")
    }
}

#[derive(Debug)]
pub struct ScheduledVerses<'a> {
    date: NaiveDate,
    verses: Vec<Verse<'a>>,
}
impl<'a> ScheduledVerses<'a> {
    pub fn new(
        date: &str,
        verses: impl IntoIterator<Item = &'a VerseEntry> + 'a,
    ) -> AnyResult<Self> {
        let date = NaiveDate::parse_from_str(date, FMT)?;
        let verses = verses
            .into_iter()
            .map(|verse| verse.calculate_relative(date))
            .collect();
        Ok(Self { date, verses })
    }

    pub fn monthly_schedule(&'a self) -> VersesForAMonth<'a> {
        VersesForAMonth::new(&self.verses)
    }

    pub fn current_week_offset(&self) -> usize {
        self.verses
            .first()
            .map(|v| {
                if v.weeks_in < 0 {
                    0
                } else {
                    (v.weeks_in % 4) as usize
                }
            })
            .unwrap_or(0)
    }

    pub fn for_today(&'a self) -> VersesForADay<'a> {
        let week = self.current_week_offset();
        let m = self.monthly_schedule();
        let week = m.weeks.get(week);

        let result = week
            .map(|week| {
                week.days
                    .get(self.date.weekday().num_days_from_sunday() as usize)
                    .cloned()
            })
            .flatten()
            .unwrap_or_default();

        result
    }
}

fn main() -> AnyResult<()> {
    // let date = "2025-07-06";
    let date = "2033-02-06";

    let references = std::fs::read_to_string(
        "/home/dgmastertemple/Development/rust/scripture_retention_algorithm/input.txt",
    )?
    .lines()
    .filter_map(|line| {
        line.trim()
            .split_once(" | ")
            .and_then(|(date, verse)| VerseEntry::new(date, verse).ok())
    })
    .collect_vec();

    // let list = VerseList::new(date, references)?;
    // let verses = list.relative_verses();

    // let verses = ScheduledVerses::new("2025-06-28", &references)?;
    // println!("{}", verses.monthly_schedule().stats());
    //
    // let verses = ScheduledVerses::new("2025-07-06", &references)?;
    // println!("{}", verses.monthly_schedule().stats());
    //
    // let verses = ScheduledVerses::new("2025-07-13", &references)?;
    // println!("{}", verses.monthly_schedule().stats());
    // dbg!(&verses.for_today());

    // let days = vec!["2025-08-24"];
    let days = std::fs::read_to_string(
        "/home/dgmastertemple/Development/rust/scripture_retention_algorithm/days.txt",
    )?
    .lines()
    .map(|l| l.trim().to_string())
    .collect_vec();

    for day in days {
        let verses = ScheduledVerses::new(day.as_str(), &references)?;
        // dbg!(&verses.for_today());
        println!("## {}\n\n{}\n", day, &verses.for_today().data());
    }

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn idk() -> AnyResult<()> {
//     }
// }
