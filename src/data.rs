use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};

use std::path::Path;

type Result<T> = ::std::result::Result<T, rusqlite::Error>;

pub struct Series {
    pub id: i32,
    pub name: String,
    pub unit: String,
    pub measurements: Vec<Measurement>
}

pub struct Measurement {
    pub timestamp: u32,
    pub run: u32,
    pub measurement: f64
}

pub struct DataSource {
    conn: Connection
}

impl DataSource {
    pub fn new<P: AsRef<Path>>(data_path: P, source: P) -> Result<Self> {
        let file = data_path.as_ref().join(source);
        let conn = Connection::open(file)?;

        // This could be smarter.. but meh. TODO future me.
        conn.execute(
            "create table if not exists series (
              id integer primary key,
              name text,
              unit text
            )", NO_PARAMS
        )?;
        conn.execute(
            "create table if not exists measurement (
              series integer,
              timestamp integer,
              run integer,
              measurement real,
              primary key(series, run),
              foreign key(series) references series(id)
            );", NO_PARAMS
        )?;

        Ok(DataSource { conn })
    }
    
    pub fn series(&self, series: &str, points: u8) -> Result<Option<Series>> {
        let mut series = match self.get_series(series)? {
            Some(s) => s,
            None => { return Ok(None); }
        };

        let mut sth = self.conn.prepare(
            "select timestamp, run, measurement
             from measurement
             where series = ?1
             order by timestamp desc limit ?2"
        )?;

        let result = sth.query_map(
            &[&series.id as &ToSql, &points],
            |row| Measurement {
                timestamp: row.get(0),
                run: row.get(1),
                measurement: row.get(2)
            })?;

        result
            .for_each(|m| series.measurements.push(m.unwrap()));

        Ok(Some(series))
    }

    pub fn measure(&self, series: &str, value: f64, create: bool) -> Result<usize> {
        let series = match self.get_series(series)? {
            Some(s) => s,
            None => {
                if create {
                    self.create_series(series, "")?
                } else {
                    eprintln!("Invalid series, use -c to force auto-creation");
                    ::std::process::exit(1);
                }
            }
        };

        self.conn.execute(
            "insert into measurement values (
              ?1,
              strftime('%s','now'),
              (select coalesce(max(run), 0) from measurement where series = ?2) + 1,
              ?3
            )",
            &[&series.id as &ToSql, &series.id, &value]
        )
    }

    pub fn create_series(&self, name: &str, unit: &str) -> Result<Series> {
        let mut ins = self.conn.prepare("insert into series (name, unit) values (?1, ?2)")?;
        let series_id = ins.insert(&[&name as &ToSql, &unit])?;
        let series_id = series_id as u32;

        let mut sth = self.conn.prepare("select id, name, unit from series where id = ?1")?;

        sth.query_row(
            &[&series_id as &ToSql],
            |row| Ok(
                Series {
                    id: row.get(0),
                    name: row.get(1),
                    unit: row.get(2),
                    measurements: vec![]
                }
            )
        )?
    }

    pub fn delete_series(&self, series: &str) -> Result<usize> {
        let series = match self.get_series(series)? {
            Some(s) => s,
            None => {
                eprintln!("Series not found");
                ::std::process::exit(1);
            }
        };

        let measurements = self.conn.execute(
            "delete from measurement where series = ?1",
            &[&series.id as &ToSql]
        )?;

        self.conn.execute(
            "delete from series where id = ?1",
            &[&series.id as &ToSql]
        )?;

        Ok(measurements)
    }

    pub fn list_series(&self) -> Result<Vec<Series>> {
        let mut sth = self.conn.prepare("select id, name, unit from series")?;

        let list = sth.query_map(
            NO_PARAMS,
            |row| {
                Series {
                    id: row.get(0),
                    name: row.get(1),
                    unit: row.get(2),
                    measurements: vec![]
                }
            })?
            .map(|s| s.unwrap())
            .collect::<Vec<_>>();

        Ok(list)
    }

    fn get_series(&self, series: &str) -> Result<Option<Series>> {
        let mut sth = self.conn.prepare(
            "select * from series where name = ?1",
        )?;

        let result = sth.query_row(
            &[&series as &ToSql],
            |row| Series {
                id: row.get(0),
                name: row.get(1),
                unit: row.get(2),
                measurements: vec![]
            }
        );

        // trick to get our a Result<Option<T>>
        match result {
            Ok(s) => Ok(Some(s)),
            Err(_) => Ok(None)
        }
    }
}
