use clap::Parser;
use lunais::disruption_calendar::generate_ical;
use lunais::timezone_pair::TimezonePair;

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
    if let Ok(tzp) = TimezonePair::try_from(format!("{0}/{1}", cli.timezone_1, cli.timezone_2)) {
        let d = tzp.get_disruption_dates(year);
        let i = generate_ical(&d);
        println!("{i}");
    } else {
        println!("Incorrect tzs: {} {}", cli.timezone_1, cli.timezone_2);
    };
}
