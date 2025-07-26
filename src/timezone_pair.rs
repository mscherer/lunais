use chrono::Datelike;
use chrono::TimeZone;
use chrono::Timelike;
use chrono::naive::NaiveDate;
use chrono_tz::Tz;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum DisruptionDate {
    DSTChaosPeriod(NaiveDate, NaiveDate),
    DSTPermanentChange(NaiveDate),
}

#[derive(PartialEq, Debug)]
pub struct TimezonePair {
    tzs: [Tz; 2],
}

pub fn parse_tz(paths: Vec<&str>) -> Option<TimezonePair> {
    let mut prefix = None;
    let mut res = Vec::new();

    // make sure we do not do a loop if the result is obviously
    // wrong (small protection against DoS)
    if paths.len() > 4 {
        return None
    }

    for item in paths {
        if prefix.is_none() {
            match item.parse() {
                Ok(tz) => res.push(tz),
                Err(_) => prefix = Some(item),
            }
        } else {
            match format!("{}/{}", prefix.unwrap(), item).parse() {
                Ok(tz) => {
                    res.push(tz);
                    prefix = None;
                }
                Err(_) => break,
            }
        }
    }

    if res.len() == 2 && prefix.is_none() {
        Some(TimezonePair::new(res[0], res[1]))
    } else {
        None
    }
}

impl TimezonePair {
    pub fn new(tz1: Tz, tz2: Tz) -> Self {
        Self { tzs: [tz1, tz2] }
    }

    pub fn get_disruption_dates(&self, year: i32) -> Vec<DisruptionDate> {
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
    use crate::timezone_pair::DisruptionDate;
    use crate::timezone_pair::parse_tz;
    use chrono::NaiveDate;
    use chrono_tz::Tz;

    #[test]
    fn test_parse_tz() {
        // fail
        for testcase in [
            "UTC",
            "UTC/gmt",
            "UTC/GMT/plop",
            "UTC/GMT/America/Paris",
            "UTC/GMT/America/Paris/coin",
            "Asia/////Tokyo/Europe/Berlin",
            "Asia/Tokyo/Europe/Berlin///",
            "//Asia/Tokyo/Europe/Berlin",
        ] {
            let r = parse_tz(testcase.split('/').collect());
            assert_eq!(r, None);
        }

        // ok
        let utc_tz: Tz = "UTC".parse().expect("is hardcoded");
        let gmt_tz: Tz = "GMT".parse().expect("is hardcoded");
        let berlin_tz: Tz = "Europe/Berlin".parse().expect("is hardcoded");
        let newyork_tz: Tz = "America/New_York".parse().expect("is hardcoded");
        let vancouver_tz: Tz = "America/Vancouver".parse().expect("is hardcoded");

        let r = parse_tz("UTC/GMT".split('/').collect()).unwrap();
        assert_eq!(r.tzs[0], utc_tz);
        assert_eq!(r.tzs[1], gmt_tz);

        let r = parse_tz("UTC/Europe/Berlin".split('/').collect()).unwrap();
        assert_eq!(r.tzs[0], utc_tz);
        assert_eq!(r.tzs[1], berlin_tz);

        let r = parse_tz("America/New_York/UTC".split('/').collect()).unwrap();
        assert_eq!(r.tzs[0], newyork_tz);
        assert_eq!(r.tzs[1], utc_tz);

        let r = parse_tz("America/Vancouver/Europe/Berlin".split('/').collect()).unwrap();
        assert_eq!(r.tzs[0], vancouver_tz);
        assert_eq!(r.tzs[1], berlin_tz);
    }

    #[test]
    fn test_disruption_date() {
        let r = parse_tz("America/Vancouver/Europe/Berlin".split('/').collect()).unwrap();
        let dd = r.get_disruption_dates(2024);

        let mut expected_res = Vec::new();
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            NaiveDate::from_ymd_opt(2024, 3, 10).expect("hardcoded"),
            NaiveDate::from_ymd_opt(2024, 3, 31).expect("hardcoded"),
        ));
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            NaiveDate::from_ymd_opt(2024, 10, 27).expect("hardcoded"),
            NaiveDate::from_ymd_opt(2024, 11, 3).expect("hardcoded"),
        ));

        assert_eq!(dd, expected_res);
    }
}
