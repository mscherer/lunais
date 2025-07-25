use chrono::Datelike;
use chrono::TimeZone;
use chrono::Timelike;
use chrono::naive::NaiveDate;
use chrono_tz::Tz;
use std::time::Duration;

#[derive(Debug)]
pub enum DisruptionDate {
    DSTChaosPeriod(NaiveDate, NaiveDate),
    DSTPermanentChange(NaiveDate),
}

#[derive(PartialEq, Debug)]
pub struct TimezonePair {
    tzs: [Tz; 2],
}

pub fn parse_tz(paths: Vec<&str>) -> Option<TimezonePair> {
    let mut tz1: Option<Tz> = None;
    let mut tz2: Option<Tz> = None;

    if paths.len() == 2 {
        tz1 = paths[0].parse().ok();
        tz2 = paths[1].parse().ok();
    } else if paths.len() == 3 {
        tz1 = paths[0].parse().ok();
        if tz1.is_some() {
            tz2 = format!("{}/{}", paths[1], paths[2]).parse().ok();
        } else {
            tz1 = format!("{}/{}", paths[0], paths[1]).parse().ok();
            tz2 = paths[3].parse().ok();
        }
    } else if paths.len() == 4 {
        tz1 = format!("{}/{}", paths[0], paths[1]).parse().ok();
        tz2 = format!("{}/{}", paths[2], paths[3]).parse().ok();
    }

    if let Some(t1) = tz1
        && let Some(t2) = tz2
    {
        Some(TimezonePair::new(t1, t2))
    } else {
        None
    }
}

impl TimezonePair {
    pub fn new(tz1: Tz, tz2: Tz) -> Self {
        Self { tzs: [tz1, tz2] }
    }

    pub fn get_disruption_dates(self: &Self, year: i32) -> Vec<DisruptionDate> {
        let mut res = Vec::new();
        let mut dt_1 = self.tzs[0]
            .with_ymd_and_hms(year, 1, 1, 12, 0, 0)
            .single()
            .unwrap();
        let mut dt_2 = self.tzs[1]
            .with_ymd_and_hms(year, 1, 1, 12, 0, 0)
            .single()
            .unwrap();
        // assume that DST is 1h
        // use wrapping_sub to avoid panic at runtime in debug
        let new_year_offset = dt_1.hour().wrapping_sub(dt_2.hour());
        let mut change_date: Option<NaiveDate> = None;
        // use hour, because offset is making borrow checker unhappy
        while dt_1.date_naive().year() < year + 1 {
            dt_1 += Duration::from_secs(60 * 60 * 24);
            dt_2 += Duration::from_secs(60 * 60 * 24);
            let offset = dt_1.hour().wrapping_sub(dt_2.hour());

            if offset != new_year_offset {
                if change_date.is_none() {
                    change_date = Some(dt_1.date_naive())
                }
            } else if let Some(d) = change_date {
                res.push(DisruptionDate::DSTChaosPeriod(d, dt_1.date_naive()));
                change_date = None;
            }
        }
        if let Some(d) = change_date {
            res.push(DisruptionDate::DSTPermanentChange(d))
        }

        res
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_parse_tz() {
        use crate::timezone_pair::parse_tz;

        // fail
        let r = parse_tz("UTC".split('/').collect());
        assert_eq!(r, None);

        let r = parse_tz("UTC/gmt".split('/').collect());
        assert_eq!(r, None);

        let r = parse_tz("UTC/GMT/plop".split('/').collect());
        assert_eq!(r, None);

        let r = parse_tz("UTC/GMT/America/Paris".split('/').collect());
        assert_eq!(r, None);

        let r = parse_tz("UTC/GMT/America/Paris/coin".split('/').collect());
        assert_eq!(r, None);

        // ok
        // TODO better test (like check the results)
        let r = parse_tz("UTC/GMT".split('/').collect());
        assert_ne!(r, None);

        let r = parse_tz("UTC/Europe/Berlin".split('/').collect());
        assert_ne!(r, None);

        let r = parse_tz("America/Vancouver/Europe/Berlin".split('/').collect());
        assert_ne!(r, None);
    }
}
