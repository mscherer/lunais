use chrono_tz::Tz;
use clap::Parser;
use lunais::disruption_calendar::generate_ical;
use lunais::disruption_time::get_disruption_dates;
use std::str::FromStr;
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    timezone_1: String,
    timezone_2: String,

    #[arg(short, long)]
    year: Option<i32>,
}

fn main() {
    let cli = Cli::parse();

    let year = cli.year.unwrap_or(2025);
    if let Ok(tz_1) = Tz::from_str(cli.timezone_1.as_str()) {
        if let Ok(tz_2) = Tz::from_str(cli.timezone_2.as_str()) {
            let d = get_disruption_dates(year, &tz_1, &tz_2);
            let i = generate_ical(&d);
            println!("{i}");
        } else {
            println!("Incorrect tz: {}", cli.timezone_2);
        };
    } else {
        println!("Incorrect tz: {}", cli.timezone_1);
    };
}
