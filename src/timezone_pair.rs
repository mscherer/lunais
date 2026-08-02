use jiff::civil::{Date, date};
use jiff::tz::TimeZone as Tz;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, PartialEq, Serialize)]
pub enum DisruptionDate {
    DSTChaosPeriod(Date, Date),
    DSTPermanentChange(Date),
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
        match Tz::get(&prefix) {
            Ok(tz) => {
                res.push(tz);
                prefix.clear();
            }
            Err(_) => prefix.push('/'),
        }
    }

    if res.len() == 2 && prefix.is_empty() {
        Some(TimezonePair::new(res[0].clone(), res[1].clone()))
    } else {
        None
    }
}

impl TimezonePair {
    pub fn new(tz1: Tz, tz2: Tz) -> Self {
        Self { tzs: [tz1, tz2] }
    }

    pub fn get_disruption_dates(&self, year: i16) -> Vec<DisruptionDate> {
        let mut res = Vec::new();
        // use 3h00 to avoid side effect of midnight and day change
        // and 3h is usually the time where a change have been enacted
        // (unless exceptions, looking at you Australia/Lord_howe)
        let mut dt_1 = date(year, 1, 1)
            .at(3, 0, 0, 0)
            .to_zoned(self.tzs[0].clone())
            .unwrap();
        let mut dt_2 = dt_1.with_time_zone(self.tzs[1].clone());
        // assume that DST is at least 1h, even if this not always true:
        // https://lists.iana.org/hyperkitty/list/tz@iana.org/thread/LK7QY5M7Q2IWXOICIVYXCBXJF2NKX66B/
        // use wrapping_sub to avoid panic at runtime in debug
        let new_year_offset = (dt_1.hour() as i32 * 60 + dt_1.minute() as i32)
            .wrapping_sub(dt_2.hour() as i32 * 60 + dt_2.minute() as i32);
        let mut change_date: Option<Date> = None;
        // use hour, because offset is making borrow checker unhappy
        while dt_1.year() < year + 1 {
            dt_1 += Duration::from_secs(60 * 60 * 24);
            dt_2 += Duration::from_secs(60 * 60 * 24);
            let offset = (dt_1.hour() as i32 * 60 + dt_1.minute() as i32)
                .wrapping_sub(dt_2.hour() as i32 * 60 + dt_2.minute() as i32);

            if offset != new_year_offset {
                if change_date.is_none() {
                    change_date = Some(dt_1.date())
                }
            } else if let Some(d) = change_date {
                res.push(DisruptionDate::DSTChaosPeriod(d, dt_1.date()));
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
    use jiff::civil::date;
    use jiff::tz::TimeZone as Tz;

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
            // this test was removed because it work with jiff
            //"UTC/gmt",
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
        let utc_tz: Tz = Tz::get("UTC").expect("is hardcoded");
        let gmt_tz: Tz = Tz::get("GMT").expect("is hardcoded");
        let berlin_tz: Tz = Tz::get("Europe/Berlin").expect("is hardcoded");
        let newyork_tz: Tz = Tz::get("America/New_York").expect("is hardcoded");
        let vancouver_tz: Tz = Tz::get("America/Vancouver").expect("is hardcoded");
        let indianapolis_tz: Tz = Tz::get("America/Indiana/Indianapolis").expect("is hardcoded");
        let buenos_aires_tz: Tz = Tz::get("America/Argentina/Buenos_Aires").expect("is hardcoded");

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
        let dd = r.get_disruption_dates(2025);

        let mut expected_res = Vec::new();
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            date(2025, 3, 9),
            date(2025, 3, 30),
        ));
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            date(2025, 10, 26),
            date(2025, 11, 2),
        ));

        assert_eq!(dd, expected_res);
    }

    #[test]
    fn test_dst_half_hour() {
        // Norfolk and Lord How change at the same time
        // but Lord Howe do only 30 minutes
        // in 2025, that's on 2025-04-06 and 2025-10-05
        let r = TimezonePair::try_from("Australia/Lord_Howe/Pacific/Norfolk").unwrap();
        let dd = r.get_disruption_dates(2025);

        let mut expected_res = Vec::new();
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            date(2025, 4, 6),
            date(2025, 10, 5),
        ));

        assert_eq!(dd, expected_res);
    }

    #[test]
    fn test_dst_2_hours_europe() {
        // Troll, a station in the antartica use a 2h DST
        // it change at the same time as Paris, at least in 2025, but
        // it change with 2h where Paris do 1h
        let r = TimezonePair::try_from("Antarctica/Troll/Europe/Paris").unwrap();
        let dd = r.get_disruption_dates(2025);

        let mut expected_res = Vec::new();
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            date(2025, 3, 30),
            date(2025, 10, 26),
        ));

        assert_eq!(dd, expected_res);
    }

    #[test]
    fn test_dst_2_hours_usa() {
        // Troll, a station in the antartica use a 2h DST
        // it change at the same time as Paris, at least in 2025,
        // and so at a different time than in NY, who change earlier in 2025
        // that's just one big period of disruption, while it could be 3, depending
        // on how we see things
        let r = TimezonePair::try_from("Antarctica/Troll/America/New_York").unwrap();
        let dd = r.get_disruption_dates(2025);

        let mut expected_res = Vec::new();
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            date(2025, 3, 10),
            date(2025, 11, 3),
        ));

        assert_eq!(dd, expected_res);
    }

    #[test]
    fn test_tz_half_hour_offset() {
        // India is on UTC+5h30 all year long, Pakistan is UTC+5
        // none observe DST as of 2025, but Pakistan tested it until 2009
        let r = TimezonePair::try_from("Asia/Calcutta/Asia/Karachi").unwrap();
        let dd = r.get_disruption_dates(2008);

        let mut expected_res = Vec::new();
        expected_res.push(DisruptionDate::DSTChaosPeriod(
            date(2008, 6, 1),
            date(2008, 11, 1),
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
    fn test_tz_order_big_diff() {
        // It also go on a different day for Vancouver and Tokyo because there is more than 12 hours
        // between them
        let r1 = TimezonePair::try_from("America/Vancouver/Asia/Tokyo").unwrap();
        let r2 = TimezonePair::try_from("Asia/Tokyo/America/Vancouver").unwrap();

        let i = 2025;
        assert_ne!(r1.get_disruption_dates(i), r2.get_disruption_dates(i));
    }

    #[test]
    fn test_tz_order_medium_diff() {
        // technically, the time change on Saturday afternoon from NY since it change on
        // Dublin night, so not the same day depending on the 1st argument
        let r1 = TimezonePair::try_from("America/New_York/Europe/Dublin").unwrap();
        let r2 = TimezonePair::try_from("Europe/Dublin/America/New_York").unwrap();

        let i = 2025;
        assert_ne!(r1.get_disruption_dates(i), r2.get_disruption_dates(i));
    }

    #[test]
    fn test_tz_order_small_diff() {
        // small diff should be ok (Cairo and Kyiv are on the same timezone, not same DST)
        let r1 = TimezonePair::try_from("Africa/Cairo/Europe/Kyiv").unwrap();
        let r2 = TimezonePair::try_from("Europe/Kyiv/Africa/Cairo").unwrap();

        let i = 2025;
        assert_eq!(r1.get_disruption_dates(i), r2.get_disruption_dates(i));
    }
}
