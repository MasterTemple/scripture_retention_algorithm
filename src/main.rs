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

// fn split_into_n_parts<T: Clone, const N: usize>(vec: Vec<T>) -> [Vec<T>; N] {
//     let len = vec.len();
//     let base = len / N;
//     let remainder = len % N;
//
//     let mut result = Vec::with_capacity(N);
//     let mut start = 0;
//
//     for i in 0..N {
//         let extra = if i < remainder { 1 } else { 0 };
//         let end = start + base + extra;
//         result.push(vec[start..end].to_vec());
//         start = end;
//     }
//
//     [
//         result[0],
//         result[1],
//         result[2],
//         result[3],
//         result[4],
//         result[5],
//         result[6],
//     ]
// }

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
    // reference: &'a String,
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

// pub struct Week(u32);

// pub struct Verse {
//     completion: String,
//     reference: String,
// }

#[derive(Debug)]
pub struct VersesForADay<'a> {
    daily: Vec<Verse<'a>>,
    weekly: Vec<Verse<'a>>,
    monthly: Vec<Verse<'a>>,
}

#[derive(Debug)]
pub struct VersesForAWeek<'a> {
    // days: [VersesForADay; 7],
    days: Vec<VersesForADay<'a>>,
}

impl<'a> VersesForAWeek<'a> {
    // pub fn new(verses: &'a Vec<Verse>, n: i64) -> Self {
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
    // weeks: [VersesForAWeek<'a>; 4],
    weeks: Vec<VersesForAWeek<'a>>,
}

impl<'a> VersesForAMonth<'a> {
    pub fn new(verses: &'a Vec<Verse>) -> Self {
        let weeks = (1..=4)
            .map(|n| {
                let verses = verses.iter().map(|v| v.with_offset(n)).collect_vec();
                VersesForAWeek::new(&verses, n)
            })
            .collect_vec();
        // vec![
        //     VersesForAWeek::new(verses, 1),
        //     VersesForAWeek::new(verses, 2),
        //     VersesForAWeek::new(verses, 3),
        //     VersesForAWeek::new(verses, 4),
        // ];
        Self { weeks }
        // let week1 = VersesForAWeek { days: vec![] };
        // let week2 = VersesForAWeek { days: vec![] };
        // let week3 = VersesForAWeek { days: vec![] };
        // let week4 = VersesForAWeek { days: vec![] };
        // Self {
        //     weeks: [week1, week2, week3, week4],
        // }
    }
}

fn main() -> AnyResult<()> {
    let date = "2025-07-06";
    let references = vec![
        VerseEntry::new("2025-07-06", "John 1:1")?,
        VerseEntry::new("2025-07-13", "John 1:2")?,
    ];
    let list = VerseList::new(date, references)?;
    let verses = list.relative_verses();

    dbg!(VersesForAWeek::new(&verses, 1));
    dbg!(VersesForAMonth::new(&verses));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idk() -> AnyResult<()> {
        let date = "2025-07-06";
        let references = vec![
            VerseEntry::new("2025-07-06", "John 1:1")?,
            VerseEntry::new("2025-07-13", "John 1:2")?,
        ];
        let list = VerseList::new(date, references)?;
        let verses = list.relative_verses();

        VersesForAMonth::new(&verses);

        Ok(())
    }

    // #[test]
    fn daily() -> AnyResult<()> {
        let reference = "John 1:1".to_string();
        let today = NaiveDate::parse_from_str("2025-07-06", FMT)?;

        macro_rules! check_entry {
            ($date:literal, $freq:ident) => {
                assert_eq!(
                    VerseEntry::new($date, "John 1:1".to_string())?.frequency(today),
                    Frequency::$freq,
                )
            };
        }

        check_entry!("2025-07-06", Daily);
        check_entry!("2025-07-13", Daily);
        check_entry!("2025-07-20", Daily);
        check_entry!("2025-07-27", Daily);
        check_entry!("2025-08-03", Daily);
        check_entry!("2025-08-10", Daily);
        check_entry!("2025-08-17", Daily);

        check_entry!("2025-08-24", Weekly);
        check_entry!("2025-08-31", Weekly);
        check_entry!("2025-09-07", Weekly);
        check_entry!("2025-09-14", Weekly);
        check_entry!("2025-09-21", Weekly);
        check_entry!("2025-09-28", Weekly);
        check_entry!("2025-10-05", Weekly);
        check_entry!("2025-10-12", Weekly);
        check_entry!("2025-10-19", Weekly);
        check_entry!("2025-10-26", Weekly);
        check_entry!("2025-11-02", Weekly);
        check_entry!("2025-11-09", Weekly);
        check_entry!("2025-11-16", Weekly);
        check_entry!("2025-11-23", Weekly);
        check_entry!("2025-11-30", Weekly);
        check_entry!("2025-12-07", Weekly);
        check_entry!("2025-12-14", Weekly);
        check_entry!("2025-12-21", Weekly);
        check_entry!("2025-12-28", Weekly);
        check_entry!("2026-01-04", Weekly);
        check_entry!("2026-01-11", Weekly);
        check_entry!("2026-01-18", Weekly);
        check_entry!("2026-01-25", Weekly);
        check_entry!("2026-02-01", Weekly);
        check_entry!("2026-02-08", Weekly);
        check_entry!("2026-02-15", Weekly);
        check_entry!("2026-02-22", Weekly);
        check_entry!("2026-03-01", Weekly);

        check_entry!("2026-03-08", Monthly);
        check_entry!("2026-03-15", Monthly);
        check_entry!("2026-03-22", Monthly);
        check_entry!("2026-03-29", Monthly);
        check_entry!("2026-04-05", Monthly);
        check_entry!("2026-04-12", Monthly);
        check_entry!("2026-04-19", Monthly);
        check_entry!("2026-04-26", Monthly);
        check_entry!("2026-05-03", Monthly);
        check_entry!("2026-05-10", Monthly);
        check_entry!("2026-05-17", Monthly);
        check_entry!("2026-05-24", Monthly);
        check_entry!("2026-05-31", Monthly);
        check_entry!("2026-06-07", Monthly);
        check_entry!("2026-06-14", Monthly);
        check_entry!("2026-06-21", Monthly);
        check_entry!("2026-06-28", Monthly);
        check_entry!("2026-07-05", Monthly);
        check_entry!("2026-07-12", Monthly);
        check_entry!("2026-07-19", Monthly);
        check_entry!("2026-07-26", Monthly);
        check_entry!("2026-08-02", Monthly);
        check_entry!("2026-08-09", Monthly);
        check_entry!("2026-08-16", Monthly);
        check_entry!("2026-08-23", Monthly);
        check_entry!("2026-08-30", Monthly);
        check_entry!("2026-09-06", Monthly);
        check_entry!("2026-09-13", Monthly);
        check_entry!("2026-09-20", Monthly);
        check_entry!("2026-09-27", Monthly);
        check_entry!("2026-10-04", Monthly);
        check_entry!("2026-10-11", Monthly);
        check_entry!("2026-10-18", Monthly);
        check_entry!("2026-10-25", Monthly);
        check_entry!("2026-11-01", Monthly);
        check_entry!("2026-11-08", Monthly);
        check_entry!("2026-11-15", Monthly);
        check_entry!("2026-11-22", Monthly);
        check_entry!("2026-11-29", Monthly);
        check_entry!("2026-12-06", Monthly);
        check_entry!("2026-12-13", Monthly);
        check_entry!("2026-12-20", Monthly);
        check_entry!("2026-12-27", Monthly);
        check_entry!("2027-01-03", Monthly);
        check_entry!("2027-01-10", Monthly);
        check_entry!("2027-01-17", Monthly);
        check_entry!("2027-01-24", Monthly);
        check_entry!("2027-01-31", Monthly);
        check_entry!("2027-02-07", Monthly);
        check_entry!("2027-02-14", Monthly);
        check_entry!("2027-02-21", Monthly);
        check_entry!("2027-02-28", Monthly);
        check_entry!("2027-03-07", Monthly);
        check_entry!("2027-03-14", Monthly);
        check_entry!("2027-03-21", Monthly);
        check_entry!("2027-03-28", Monthly);
        check_entry!("2027-04-04", Monthly);
        check_entry!("2027-04-11", Monthly);
        check_entry!("2027-04-18", Monthly);
        check_entry!("2027-04-25", Monthly);
        check_entry!("2027-05-02", Monthly);
        check_entry!("2027-05-09", Monthly);
        check_entry!("2027-05-16", Monthly);
        check_entry!("2027-05-23", Monthly);
        check_entry!("2027-05-30", Monthly);
        check_entry!("2027-06-06", Monthly);
        check_entry!("2027-06-13", Monthly);
        check_entry!("2027-06-20", Monthly);
        check_entry!("2027-06-27", Monthly);
        check_entry!("2027-07-04", Monthly);
        check_entry!("2027-07-11", Monthly);
        check_entry!("2027-07-18", Monthly);
        check_entry!("2027-07-25", Monthly);
        check_entry!("2027-08-01", Monthly);
        check_entry!("2027-08-08", Monthly);
        check_entry!("2027-08-15", Monthly);
        check_entry!("2027-08-22", Monthly);
        check_entry!("2027-08-29", Monthly);
        check_entry!("2027-09-05", Monthly);
        check_entry!("2027-09-12", Monthly);
        check_entry!("2027-09-19", Monthly);
        check_entry!("2027-09-26", Monthly);
        check_entry!("2027-10-03", Monthly);
        check_entry!("2027-10-10", Monthly);
        check_entry!("2027-10-17", Monthly);
        check_entry!("2027-10-24", Monthly);
        check_entry!("2027-10-31", Monthly);
        check_entry!("2027-11-07", Monthly);
        check_entry!("2027-11-14", Monthly);
        check_entry!("2027-11-21", Monthly);
        check_entry!("2027-11-28", Monthly);
        check_entry!("2027-12-05", Monthly);
        check_entry!("2027-12-12", Monthly);
        check_entry!("2027-12-19", Monthly);
        check_entry!("2027-12-26", Monthly);
        check_entry!("2028-01-02", Monthly);
        check_entry!("2028-01-09", Monthly);
        check_entry!("2028-01-16", Monthly);
        check_entry!("2028-01-23", Monthly);
        check_entry!("2028-01-30", Monthly);
        check_entry!("2028-02-06", Monthly);
        check_entry!("2028-02-13", Monthly);
        check_entry!("2028-02-20", Monthly);
        check_entry!("2028-02-27", Monthly);
        check_entry!("2028-03-05", Monthly);
        check_entry!("2028-03-12", Monthly);
        check_entry!("2028-03-19", Monthly);
        check_entry!("2028-03-26", Monthly);
        check_entry!("2028-04-02", Monthly);
        check_entry!("2028-04-09", Monthly);
        check_entry!("2028-04-16", Monthly);
        check_entry!("2028-04-23", Monthly);
        check_entry!("2028-04-30", Monthly);
        check_entry!("2028-05-07", Monthly);
        check_entry!("2028-05-14", Monthly);
        check_entry!("2028-05-21", Monthly);
        check_entry!("2028-05-28", Monthly);
        check_entry!("2028-06-04", Monthly);
        check_entry!("2028-06-11", Monthly);
        check_entry!("2028-06-18", Monthly);
        check_entry!("2028-06-25", Monthly);
        check_entry!("2028-07-02", Monthly);
        check_entry!("2028-07-09", Monthly);
        check_entry!("2028-07-16", Monthly);
        check_entry!("2028-07-23", Monthly);
        check_entry!("2028-07-30", Monthly);
        check_entry!("2028-08-06", Monthly);
        check_entry!("2028-08-13", Monthly);
        check_entry!("2028-08-20", Monthly);
        check_entry!("2028-08-27", Monthly);
        check_entry!("2028-09-03", Monthly);
        check_entry!("2028-09-10", Monthly);
        check_entry!("2028-09-17", Monthly);
        check_entry!("2028-09-24", Monthly);
        check_entry!("2028-10-01", Monthly);
        check_entry!("2028-10-08", Monthly);
        check_entry!("2028-10-15", Monthly);
        check_entry!("2028-10-22", Monthly);
        check_entry!("2028-10-29", Monthly);
        check_entry!("2028-11-05", Monthly);
        check_entry!("2028-11-12", Monthly);
        check_entry!("2028-11-19", Monthly);
        check_entry!("2028-11-26", Monthly);
        check_entry!("2028-12-03", Monthly);
        check_entry!("2028-12-10", Monthly);
        check_entry!("2028-12-17", Monthly);
        check_entry!("2028-12-24", Monthly);
        check_entry!("2028-12-31", Monthly);
        check_entry!("2029-01-07", Monthly);
        check_entry!("2029-01-14", Monthly);
        check_entry!("2029-01-21", Monthly);
        check_entry!("2029-01-28", Monthly);
        check_entry!("2029-02-04", Monthly);
        check_entry!("2029-02-11", Monthly);
        check_entry!("2029-02-18", Monthly);
        check_entry!("2029-02-25", Monthly);
        check_entry!("2029-03-04", Monthly);
        check_entry!("2029-03-11", Monthly);
        check_entry!("2029-03-18", Monthly);
        check_entry!("2029-03-25", Monthly);
        check_entry!("2029-04-01", Monthly);
        check_entry!("2029-04-08", Monthly);
        check_entry!("2029-04-15", Monthly);
        check_entry!("2029-04-22", Monthly);
        check_entry!("2029-04-29", Monthly);
        check_entry!("2029-05-06", Monthly);
        check_entry!("2029-05-13", Monthly);
        check_entry!("2029-05-20", Monthly);
        check_entry!("2029-05-27", Monthly);
        check_entry!("2029-06-03", Monthly);
        check_entry!("2029-06-10", Monthly);
        check_entry!("2029-06-17", Monthly);
        check_entry!("2029-06-24", Monthly);
        check_entry!("2029-07-01", Monthly);
        check_entry!("2029-07-08", Monthly);
        check_entry!("2029-07-15", Monthly);
        check_entry!("2029-07-22", Monthly);
        check_entry!("2029-07-29", Monthly);
        check_entry!("2029-08-05", Monthly);
        check_entry!("2029-08-12", Monthly);
        check_entry!("2029-08-19", Monthly);
        check_entry!("2029-08-26", Monthly);
        check_entry!("2029-09-02", Monthly);
        check_entry!("2029-09-09", Monthly);
        check_entry!("2029-09-16", Monthly);
        check_entry!("2029-09-23", Monthly);
        check_entry!("2029-09-30", Monthly);
        check_entry!("2029-10-07", Monthly);
        check_entry!("2029-10-14", Monthly);
        check_entry!("2029-10-21", Monthly);
        check_entry!("2029-10-28", Monthly);
        check_entry!("2029-11-04", Monthly);
        check_entry!("2029-11-11", Monthly);
        check_entry!("2029-11-18", Monthly);
        check_entry!("2029-11-25", Monthly);
        check_entry!("2029-12-02", Monthly);
        check_entry!("2029-12-09", Monthly);
        check_entry!("2029-12-16", Monthly);
        check_entry!("2029-12-23", Monthly);
        check_entry!("2029-12-30", Monthly);
        check_entry!("2030-01-06", Monthly);
        check_entry!("2030-01-13", Monthly);
        check_entry!("2030-01-20", Monthly);
        check_entry!("2030-01-27", Monthly);
        check_entry!("2030-02-03", Monthly);
        check_entry!("2030-02-10", Monthly);
        check_entry!("2030-02-17", Monthly);
        check_entry!("2030-02-24", Monthly);
        check_entry!("2030-03-03", Monthly);
        check_entry!("2030-03-10", Monthly);
        check_entry!("2030-03-17", Monthly);
        check_entry!("2030-03-24", Monthly);
        check_entry!("2030-03-31", Monthly);
        check_entry!("2030-04-07", Monthly);
        check_entry!("2030-04-14", Monthly);
        check_entry!("2030-04-21", Monthly);
        check_entry!("2030-04-28", Monthly);
        check_entry!("2030-05-05", Monthly);
        check_entry!("2030-05-12", Monthly);
        check_entry!("2030-05-19", Monthly);
        check_entry!("2030-05-26", Monthly);
        check_entry!("2030-06-02", Monthly);
        check_entry!("2030-06-09", Monthly);
        check_entry!("2030-06-16", Monthly);
        check_entry!("2030-06-23", Monthly);
        check_entry!("2030-06-30", Monthly);
        check_entry!("2030-07-07", Monthly);
        check_entry!("2030-07-14", Monthly);
        check_entry!("2030-07-21", Monthly);
        check_entry!("2030-07-28", Monthly);
        check_entry!("2030-08-04", Monthly);
        check_entry!("2030-08-11", Monthly);
        check_entry!("2030-08-18", Monthly);
        check_entry!("2030-08-25", Monthly);
        check_entry!("2030-09-01", Monthly);
        check_entry!("2030-09-08", Monthly);
        check_entry!("2030-09-15", Monthly);
        check_entry!("2030-09-22", Monthly);
        check_entry!("2030-09-29", Monthly);
        check_entry!("2030-10-06", Monthly);
        check_entry!("2030-10-13", Monthly);
        check_entry!("2030-10-20", Monthly);
        check_entry!("2030-10-27", Monthly);
        check_entry!("2030-11-03", Monthly);
        check_entry!("2030-11-10", Monthly);
        check_entry!("2030-11-17", Monthly);
        check_entry!("2030-11-24", Monthly);
        check_entry!("2030-12-01", Monthly);
        check_entry!("2030-12-08", Monthly);
        check_entry!("2030-12-15", Monthly);
        check_entry!("2030-12-22", Monthly);
        check_entry!("2030-12-29", Monthly);
        check_entry!("2031-01-05", Monthly);
        check_entry!("2031-01-12", Monthly);
        check_entry!("2031-01-19", Monthly);
        check_entry!("2031-01-26", Monthly);
        check_entry!("2031-02-02", Monthly);
        check_entry!("2031-02-09", Monthly);
        check_entry!("2031-02-16", Monthly);
        check_entry!("2031-02-23", Monthly);
        check_entry!("2031-03-02", Monthly);
        check_entry!("2031-03-09", Monthly);
        check_entry!("2031-03-16", Monthly);
        check_entry!("2031-03-23", Monthly);
        check_entry!("2031-03-30", Monthly);
        check_entry!("2031-04-06", Monthly);
        check_entry!("2031-04-13", Monthly);
        check_entry!("2031-04-20", Monthly);
        check_entry!("2031-04-27", Monthly);
        check_entry!("2031-05-04", Monthly);
        check_entry!("2031-05-11", Monthly);
        check_entry!("2031-05-18", Monthly);
        check_entry!("2031-05-25", Monthly);
        check_entry!("2031-06-01", Monthly);
        check_entry!("2031-06-08", Monthly);
        check_entry!("2031-06-15", Monthly);
        check_entry!("2031-06-22", Monthly);
        check_entry!("2031-06-29", Monthly);
        check_entry!("2031-07-06", Monthly);
        check_entry!("2031-07-13", Monthly);
        check_entry!("2031-07-20", Monthly);
        check_entry!("2031-07-27", Monthly);
        check_entry!("2031-08-03", Monthly);
        check_entry!("2031-08-10", Monthly);
        check_entry!("2031-08-17", Monthly);
        check_entry!("2031-08-24", Monthly);
        check_entry!("2031-08-31", Monthly);
        check_entry!("2031-09-07", Monthly);
        check_entry!("2031-09-14", Monthly);
        check_entry!("2031-09-21", Monthly);
        check_entry!("2031-09-28", Monthly);
        check_entry!("2031-10-05", Monthly);
        check_entry!("2031-10-12", Monthly);
        check_entry!("2031-10-19", Monthly);
        check_entry!("2031-10-26", Monthly);
        check_entry!("2031-11-02", Monthly);
        check_entry!("2031-11-09", Monthly);
        check_entry!("2031-11-16", Monthly);
        check_entry!("2031-11-23", Monthly);
        check_entry!("2031-11-30", Monthly);
        check_entry!("2031-12-07", Monthly);
        check_entry!("2031-12-14", Monthly);
        check_entry!("2031-12-21", Monthly);
        check_entry!("2031-12-28", Monthly);
        check_entry!("2032-01-04", Monthly);
        check_entry!("2032-01-11", Monthly);
        check_entry!("2032-01-18", Monthly);
        check_entry!("2032-01-25", Monthly);
        check_entry!("2032-02-01", Monthly);
        check_entry!("2032-02-08", Monthly);
        check_entry!("2032-02-15", Monthly);
        check_entry!("2032-02-22", Monthly);
        check_entry!("2032-02-29", Monthly);
        check_entry!("2032-03-07", Monthly);
        check_entry!("2032-03-14", Monthly);
        check_entry!("2032-03-21", Monthly);
        check_entry!("2032-03-28", Monthly);
        check_entry!("2032-04-04", Monthly);
        check_entry!("2032-04-11", Monthly);
        check_entry!("2032-04-18", Monthly);
        check_entry!("2032-04-25", Monthly);
        check_entry!("2032-05-02", Monthly);
        check_entry!("2032-05-09", Monthly);
        check_entry!("2032-05-16", Monthly);
        check_entry!("2032-05-23", Monthly);
        check_entry!("2032-05-30", Monthly);
        check_entry!("2032-06-06", Monthly);
        check_entry!("2032-06-13", Monthly);
        check_entry!("2032-06-20", Monthly);
        check_entry!("2032-06-27", Monthly);
        check_entry!("2032-07-04", Monthly);
        check_entry!("2032-07-11", Monthly);
        check_entry!("2032-07-18", Monthly);
        check_entry!("2032-07-25", Monthly);
        check_entry!("2032-08-01", Monthly);
        check_entry!("2032-08-08", Monthly);

        check_entry!("2032-08-15", Done);
        check_entry!("2032-08-22", Done);
        check_entry!("2032-08-29", Done);
        check_entry!("2032-09-05", Done);
        check_entry!("2032-09-12", Done);
        check_entry!("2032-09-19", Done);
        check_entry!("2032-09-26", Done);
        check_entry!("2032-10-03", Done);
        check_entry!("2032-10-10", Done);
        check_entry!("2032-10-17", Done);
        check_entry!("2032-10-24", Done);
        check_entry!("2032-10-31", Done);
        check_entry!("2032-11-07", Done);
        check_entry!("2032-11-14", Done);
        check_entry!("2032-11-21", Done);
        check_entry!("2032-11-28", Done);
        check_entry!("2032-12-05", Done);
        check_entry!("2032-12-12", Done);
        check_entry!("2032-12-19", Done);
        check_entry!("2032-12-26", Done);
        check_entry!("2033-01-02", Done);
        check_entry!("2033-01-09", Done);
        check_entry!("2033-01-16", Done);
        check_entry!("2033-01-23", Done);
        check_entry!("2033-01-30", Done);
        check_entry!("2033-02-06", Done);

        Ok(())
    }
}
