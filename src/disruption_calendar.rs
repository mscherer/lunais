use crate::timezone_pair::DisruptionDate;
use icalendar;
use icalendar::Component;
use icalendar::Event;
use icalendar::EventLike;

pub fn generate_ical(dates: &Vec<DisruptionDate>) -> icalendar::Calendar {
    let mut i = icalendar::Calendar::new();
    i.name("Timezone chaos meeting");
    for d in dates {
        match d {
            DisruptionDate::DSTChaosPeriod(s, e) => i.push(Event::new()
                .starts(*s)
                .ends(*e)
                .summary("Meeting chaos period")
                .description(
                    "Beware, meeting conflicts may happen and move around, fasten your seat belt",
                )),
            DisruptionDate::DSTPermanentChange(c) => i.push(Event::new()
                .all_day(*c)
                .summary("Permament TZ change")
                .description(
                    "The TZ is permanently changing in DST/not DST time (or there is a bug)",
                )),
        };
    }
    i.done()
}

#[cfg(test)]
mod test {
    use crate::disruption_calendar::generate_ical;
    use crate::timezone_pair::DisruptionDate;
    use chrono::naive::NaiveDate;
    use icalendar::Component;

    #[test]
    fn test_generate_ical() {
        let mut dates = Vec::new();
        dates.push(DisruptionDate::DSTChaosPeriod(
            NaiveDate::from_ymd_opt(2024, 3, 10).expect("hardcoded 10th of March"),
            NaiveDate::from_ymd_opt(2024, 3, 31).expect("hardcoded 31th of March"),
        ));
        dates.push(DisruptionDate::DSTPermanentChange(
            NaiveDate::from_ymd_opt(2024, 10, 27).expect("hardcoded date in october"),
        ));
        let cal = generate_ical(&dates);
        assert_eq!(cal.components.len(), 2);
        assert_eq!(
            cal.components
                .get(0)
                .expect("hardcoded 1st element")
                .as_event()
                .expect("hardcoded event")
                .get_summary()
                .unwrap(),
            "Meeting chaos period"
        );
        assert_eq!(
            cal.components
                .get(1)
                .expect("hardcoded 2nd element")
                .as_event()
                .expect("hardcoded event")
                .get_summary()
                .unwrap(),
            "Permament TZ change"
        );
    }
}
