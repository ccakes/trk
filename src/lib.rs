#[macro_use] extern crate log;
#[macro_use] extern crate failure;
extern crate fern;
extern crate chrono;
extern crate structopt;

#[macro_use] extern crate prettytable;
extern crate rusqlite;
extern crate termion;
extern crate textplots;
extern crate read_input;

use prettytable::Table;
use chrono::{Local, TimeZone};
use textplots::{Chart, Shape, Plot};

use std::{env, fs, io};
use std::io::{BufRead, Read};
use std::path::PathBuf;

mod cli;
mod cmd;
mod data;
mod menu;

use cli::Command;
use menu::Menu;
use data::DataSource;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Database error: {}", inner)]
    SqliteError {
        inner: rusqlite::Error
    }
}

pub fn run() -> i32 {
    let args = cli::init();

    let data_root = match args.data_root {
        Some(path) => path,
        None => {
            match env::var("HOME") {
                Ok(home) => PathBuf::from(home).join(".trk"),
                Err(_) => {
                    eprintln!("$HOME no set, use --data-root to set path");
                    std::process::exit(1);
                }
            }
        }
    };

    // Check that data_path exists, otherwise create
    match fs::metadata(&data_root) {
        Ok(md) => {
            if !md.is_dir() {
                eprintln!("{} exists but is not a directory, invalid data path", data_root.display());
                return 1;
            }
        },
        Err(ref e) if e.kind() == io::ErrorKind::PermissionDenied => {
            eprintln!("{}: Permission denied", data_root.display());
            return 1;
        },
        Err(_) => {
            if let Err(e) = fs::create_dir_all(&data_root) {
                do_error("Error creating data path", e);
                return 1;
            }
        }
    };

    let log_file = args.log_file.unwrap_or_else(|| data_root.join("trk.log") );

    // Init logger
    fern::Dispatch::new()
        .format(|out, m, r| {
            out.finish(format_args!(
                "[{}] {} {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                r.level(),
                m
            ))
        })
        .level(log::LevelFilter::Warn)
        .chain(fern::log_file(log_file).unwrap())
        .apply()
        .unwrap();

    let db = match DataSource::new(&data_root, &args.file) {
        Ok(db) => db,
        Err(e) => {
            do_error("Error initialising data source", e);
            return 1;
        }
    };

    match args.subcmd {
        Command::Plot { series, points, table } => {
            let series = match series {
                Some(s) => s,
                None => {
                    match db.list_series() {
                        Ok(list) => {
                            let list: Vec<_> = list.iter().map(|s| s.name.as_str()).collect();
                            Menu::from_vec("Select series to plot:", &list).show().to_owned()
                        },
                        Err(e) => {
                            do_error("Error getting series list", e);
                            std::process::exit(1);
                        }
                    }
                }
            };

            match db.series(&series, points) {
                Ok(Some(data)) => {
                    println!("# Series: {}\n", series);

                    let mut prepared = data.measurements.iter()
                        .rev()
                        .enumerate()
                        .map(|(i, point)| {
                            ((i) as f32, point.measurement as f32)
                        })
                        .collect::<Vec<_>>();
                    prepared.insert(0, (0.0f32, 0.0f32));

                    let x_width = prepared.len() as f32;

                    Chart::new(150, 60, 0.0, x_width)
                        .y_label(&data.unit)
                        .lineplot( Shape::Lines(prepared.as_slice()) )
                        .nice();
                    
                    if table {
                        let mut table = Table::new();
                        table.add_row(row!["#", "TIMESTAMP", "VALUE"]);

                        data.measurements.iter()
                            .enumerate()
                            .for_each(|(i, p)| {
                                let run = data.measurements.len() - i;
                                let ts = Local.timestamp(p.timestamp.into(), 0);

                                table.add_row(row![
                                    run,
                                    ts.format("%Y-%m-%d %H:%M:%S"),
                                    p.measurement
                                ]);
                            });
                        
                        println!(""); // newline
                        table.printstd();
                    }
                },
                Ok(None) => {
                    println!("Series not found");
                },
                Err(e) => {
                    do_error("Error querying series data", e);
                    return 1;
                }
            }
        },
        Command::AddBulk { create } => {
            let stdin = io::stdin();
            let stdin = stdin.lock();

            stdin.lines()
                .for_each(|line| {
                    let line = line.unwrap();

                    if line.is_empty() { return; }

                    let idx = match line.find('=') {
                        Some(i) => i,
                        None => {
                            warn!("Invalid input format: '{}'", line);
                            return;
                        }
                    };

                    let (series, value) = line.split_at(idx);
                    let value = match value.trim_start_matches('=').parse::<f64>() {
                        Ok(v) => v,
                        Err(e) => {
                            do_error("Error parsing value", e);
                            std::process::exit(1);
                        }
                    };

                    // println!("{} => {}", series, value.trim_start_matches('='));

                    match db.measure(&series, value, create) {
                        Ok(_) => {},
                        Err(e) => {
                            do_error(&format!("Error adding measurement to {}", series), e);
                            std::process::exit(1);
                        }
                    };
                });
        },
        Command::AddMeasurement { series, value, create } => {
            let value = match value {
                Some(v) => v,
                None => {
                    let stdin = io::stdin();
                    let mut stdin = stdin.lock();

                    let mut buf = String::new();
                    match stdin.read_to_string(&mut buf) {
                        Ok(_) => {
                            // println!("[D]-> |{}|", buf);
                            match buf.trim().parse::<f64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    do_error("Error parsing stdin to value", e);
                                    std::process::exit(1);
                                }
                            }
                        },
                        Err(e) => {
                            do_error("Error reading value from stdin", e);
                            std::process::exit(1);
                        }
                    }
                }
            };

            match db.measure(&series, value, create) {
                Ok(_) => {
                    debug!("Added {} to series {} in source {}", value, series, args.file.display());
                },
                Err(e) => {
                    do_error("Error adding measurement to series", e);
                    return 1;
                }
            };
        },
        Command::AddSeries { name, unit } => cmd::series::create(&db, name, unit),
        Command::DeleteSeries { series } => cmd::series::delete(&db, series)
    }

    
    0
}

pub fn do_error<E: std::error::Error>(msg: &str, e: E) {
    error!("{}", &format!("{}: {}", msg, e));
    eprintln!("{}", &format!("{}! Check log file for detail.", msg));
}
