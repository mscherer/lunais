use crate::disruption_time::DisruptionDate;
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
