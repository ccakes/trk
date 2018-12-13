use structopt::StructOpt;

use std::path::PathBuf;
use std::num::ParseFloatError;

use std::str::FromStr;

fn parse_measurement(input: &str) -> Result<f64, ParseFloatError> {
    f64::from_str(input)
}

#[derive(StructOpt)]
pub enum Command {
    /// Add a new measurement to the given series
    #[structopt(name = "add")]
    AddMeasurement {
        #[structopt(short = "s", long = "series")]
        series: String,

        #[structopt(parse(try_from_str = "parse_measurement"))]
        value: Option<f64>,

        /// Auto-create the series if it doesn't exist
        #[structopt(short = "c")]
        create: bool
    },

    /// Slurp in series:val pairs from stdin
    #[structopt(name = "bulk")]
    AddBulk {
        /// Auto-create series if they doesn't exist
        #[structopt(short = "c")]
        create: bool
    },

    /// Add a series
    #[structopt(name = "add-series")]
    AddSeries {
        #[structopt(short = "n", long = "name")]
        name: Option<String>,

        /// Unit for values (eg ms, bps, pps)
        #[structopt(short = "u", long = "unit")]
        unit: Option<String>
    },

    /// Delete a series
    #[structopt(name = "delete-series")]
    DeleteSeries {
        #[structopt(short = "s", long = "series")]
        series: Option<String>
    },

    /// Plot a series
    #[structopt(name = "plot")]
    Plot {
        #[structopt(short = "s", long = "series")]
        series: Option<String>,

        /// Number of points to plot (default 50)
        #[structopt(short = "p", default_value = "50")]
        points: u8,

        /// Show a table as well
        #[structopt(short = "t", long = "table")]
        table: bool,
    }
}

#[derive(StructOpt)]
#[structopt(name = "trk", about = "Simple CLI-based metric tracker and plotter")]
pub struct Config {
    /// Path to store data
    #[structopt(short = "d", long = "data-root", parse(from_os_str))]
    pub data_root: Option<PathBuf>,

    /// Source name
    #[structopt(short = "f", long = "data-file", default_value = "default", parse(from_os_str))]
    pub file: PathBuf,

    /// Log file
    #[structopt(short = "l", long = "log", parse(from_os_str))]
    pub log_file: Option<PathBuf>,

    #[structopt(subcommand)]
    pub subcmd: Command
}

pub fn init() -> Config {
    Config::from_args()
}
