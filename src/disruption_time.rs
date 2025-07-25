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

/*
 * #[derive(Debug)]
pub struct DSTChaosPeriod {
    start: NaiveDate,
    end: NaiveDate,
}

#[derive(Debug)]
pub struct DSTPermanentChange(NaiveDate);
*/
// faire une boucle sur la date avec 2 TZ
// calculer l'offset le 1er janvier
// faire un vec de DisruptionDate
// si l'offest n'est pas 0, ajouter dans une var
// si l'offset revient à 0, ajouter dans le vec un DSTChaosPeriod avec la var + la date de la
// boucle
// si on finit l'année avec un offset != 0, ajouter la date comme DSTPermanentChange dans Vec
// renvoyer le vec

pub fn get_disruption_dates(year: i32, tz_1: &Tz, tz_2: &Tz) -> Vec<DisruptionDate> {
    let mut res = Vec::new();
    let mut dt_1 = tz_1
        .with_ymd_and_hms(year, 1, 1, 12, 0, 0)
        .single()
        .unwrap();
    let mut dt_2 = tz_2
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

// convert to ical
//
// TODO add tests using a year in the past with know
//
//  [DSTChaosPeriod(2025-03-09, 2025-03-30), DSTChaosPeriod(2025-10-26, 2025-11-02)]
