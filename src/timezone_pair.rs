use chrono::Datelike;
use chrono::TimeZone;
use chrono::Timelike;
use chrono::naive::NaiveDate;
use chrono_tz::Tz;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum DisruptionDate {
    DSTChaosPeriod(NaiveDate, NaiveDate),
    DSTPermanentChange(NaiveDate),
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct TimezonePair {
    tzs: [Tz; 2],
}

fn parse_tz(paths: Vec<&str>) -> Option<TimezonePair> {
    let mut prefix = String::from("");
    let mut res = Vec::new();

    // make sure we do not do a loop if the result is obviously
    // wrong (small protection against DoS)
    // 6 is the maximum for 2 TZs
    if paths.len() > 6 {
        return None;
    }

    for item in paths {
        prefix.push_str(item);
        match prefix.parse() {
            Ok(tz) => {
                res.push(tz);
                prefix.clear();
            }
            Err(_) => prefix.push('/'),
        }
    }

    if res.len() == 2 && prefix.is_empty() {
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
        let mut dt_2 = dt_1.with_timezone(&self.tzs[1]);
        // assume that DST is at least 1h, even if this not always true:
        // https://lists.iana.org/hyperkitty/list/tz@iana.org/thread/LK7QY5M7Q2IWXOICIVYXCBXJF2NKX66B/
        // use wrapping_sub to avoid panic at runtime in debug
        let new_year_offset =
            (dt_1.hour() * 60 + dt_1.minute()).wrapping_sub(dt_2.hour() * 60 + dt_2.minute());
        let mut change_date: Option<NaiveDate> = None;
        // use hour, because offset is making borrow checker unhappy
        while dt_1.date_naive().year() < year + 1 {
            dt_1 += Duration::from_secs(60 * 60 * 24);
            dt_2 += Duration::from_secs(60 * 60 * 24);
            let offset =
                (dt_1.hour() * 60 + dt_1.minute()).wrapping_sub(dt_2.hour() * 60 + dt_2.minute());

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

impl TryFrom<String> for TimezonePair {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        TimezonePair::try_from(value.as_ref())
    }
}

impl TryFrom<&str> for TimezonePair {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_tz(value.split('/').collect()).ok_or("Invalid string")
    }
}

#[cfg(test)]
mod test {
    use crate::timezone_pair::DisruptionDate;
    use crate::timezone_pair::TimezonePair;
    use crate::timezone_pair::parse_tz;
    use chrono::NaiveDate;
    use chrono_tz::Tz;

    #[test]
    fn test_try_from() {
        let r = TimezonePair::try_from("UTC/UTC");
        assert_eq!(r.is_ok(), true);

        let r = TimezonePair::try_from("UTC/UTC".to_owned());
        assert_eq!(r.is_ok(), true);
    }

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
        let indianapolis_tz: Tz = "America/Indiana/Indianapolis"
            .parse()
            .expect("is hardcoded");
        let buenos_aires_tz: Tz = "America/Argentina/Buenos_Aires"
            .parse()
            .expect("is hardcoded");

        let r = TimezonePair::try_from("UTC/GMT").unwrap();
        assert_eq!(r.tzs[0], utc_tz);
        assert_eq!(r.tzs[1], gmt_tz);

        let r = TimezonePair::try_from("UTC/Europe/Berlin").unwrap();
        assert_eq!(r.tzs[0], utc_tz);
        assert_eq!(r.tzs[1], berlin_tz);

        let r = TimezonePair::try_from("America/New_York/UTC").unwrap();
        assert_eq!(r.tzs[0], newyork_tz);
        assert_eq!(r.tzs[1], utc_tz);

        let r = TimezonePair::try_from("America/Vancouver/Europe/Berlin").unwrap();
        assert_eq!(r.tzs[0], vancouver_tz);
        assert_eq!(r.tzs[1], berlin_tz);

        let r = TimezonePair::try_from("America/Vancouver/America/Indiana/Indianapolis").unwrap();
        assert_eq!(r.tzs[0], vancouver_tz);
        assert_eq!(r.tzs[1], indianapolis_tz);

        let r =
            TimezonePair::try_from("America/Argentina/Buenos_Aires/America/Indiana/Indianapolis")
                .unwrap();
        assert_eq!(r.tzs[0], buenos_aires_tz);
        assert_eq!(r.tzs[1], indianapolis_tz);
    }

    #[test]
    fn test_disruption_date() {
        let r = TimezonePair::try_from("America/Vancouver/Europe/Berlin").unwrap();
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

    #[test]
    fn test_dst_half_hour() {
        // Norfolk and Lord How change at the same time
        // but Lord Howe do only 30 minutes
        // in 2024, that's on 2024-04-07 and 2024-10-06
        let r = TimezonePair::try_from("Australia/Lord_Howe/Pacific/Norfolk").unwrap();
        let dd = r.get_disruption_dates(2024);

        let mut expected_res = Vec::new();
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            NaiveDate::from_ymd_opt(2024, 4, 7).expect("hardcoded"),
            NaiveDate::from_ymd_opt(2024, 10, 6).expect("hardcoded"),
        ));

        assert_eq!(dd, expected_res);
    }

    #[test]
    fn test_dst_2_hours_europe() {
        // Troll, a station in the antartica use a 2h DST
        // it change at the same time as Paris, at least in 2024, but
        // it change with 2h where Paris do 1h
        let r = TimezonePair::try_from("Antarctica/Troll/Europe/Paris").unwrap();
        let dd = r.get_disruption_dates(2024);

        let mut expected_res = Vec::new();
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            NaiveDate::from_ymd_opt(2024, 3, 31).expect("hardcoded"),
            NaiveDate::from_ymd_opt(2024, 10, 27).expect("hardcoded"),
        ));

        assert_eq!(dd, expected_res);
    }

    #[test]
    fn test_dst_2_hours_usa() {
        // Troll, a station in the antartica use a 2h DST
        // it change at the same time as Paris, at least in 2024,
        // and so at a different time than in NY
        // that's just one big period of disruption, while it could be 3, depending
        // on how we see things
        let r = TimezonePair::try_from("Antarctica/Troll/America/New_York").unwrap();
        let dd = r.get_disruption_dates(2024);

        let mut expected_res = Vec::new();
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            NaiveDate::from_ymd_opt(2024, 3, 10).expect("hardcoded"),
            NaiveDate::from_ymd_opt(2024, 11, 3).expect("hardcoded"),
        ));

        assert_eq!(dd, expected_res);
    }

    #[test]
    fn test_tz_half_hour_offset() {
        // India is on UTC+5h30 all year long, Pakistan is UTC+5
        // none observe DST as of 2024, but Pakistan tested it until 2009
        let r = TimezonePair::try_from("Asia/Calcutta/Asia/Karachi").unwrap();
        let dd = r.get_disruption_dates(2008);

        let mut expected_res = Vec::new();
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            NaiveDate::from_ymd_opt(2008, 6, 1).expect("hardcoded"),
            NaiveDate::from_ymd_opt(2008, 11, 1).expect("hardcoded"),
        ));

        assert_eq!(dd, expected_res);
    }

    #[test]
    fn test_utc_plus_14() {
        // Since Kiritimati is always UTC+14, and Atka is UTC+10 with DST, the DST change do not
        // happen on the same calendar day
        let r1 = TimezonePair::try_from("Pacific/Kiritimati/America/Atka").unwrap();
        let r2 = TimezonePair::try_from("America/Atka/Pacific/Kiritimati").unwrap();

        let i = 2025;
        assert_ne!(r1.get_disruption_dates(i), r2.get_disruption_dates(i));
    }

    #[test]
    fn test_tz_order() {
        // It also fail with Vancouver and Tokyo because there is more than 12 hours
        // between them
        let r1 = TimezonePair::try_from("America/Vancouver/Asia/Tokyo").unwrap();
        let r2 = TimezonePair::try_from("Asia/Tokyo/America/Vancouver").unwrap();

        let i = 2025;
        assert_ne!(r1.get_disruption_dates(i), r2.get_disruption_dates(i));

        // same day for Berlin and Vancouver
        let r1 = TimezonePair::try_from("America/Vancouver/Europe/Berlin").unwrap();
        let r2 = TimezonePair::try_from("Europe/Berlin/America/Vancouver").unwrap();
        assert_eq!(r1.get_disruption_dates(i), r2.get_disruption_dates(i));
    }
}
