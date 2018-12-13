use do_error;
use menu::Menu;
use data::DataSource;

use read_input::input_new;

pub fn create(db: &DataSource, name: Option<String>, unit: Option<String>) {
    let name = name.unwrap_or_else(|| {
        input_new().msg("Series Name: ").get()
    });

    let unit = unit.unwrap_or_else(|| {
        input_new().msg("Input Unit (eg ms, bps): ").get()
    });

    match db.create_series(&name, &unit) {
        Ok(series) => println!("Created {}", series.name),
        Err(e) => {
            do_error("Error creating series", e);
            std::process::exit(1);
        }
    };
}

pub fn delete(db: &DataSource, name: Option<String>) {
    let name = match name {
        Some(s) => s,
        None => {
            match db.list_series() {
                Ok(list) => {
                    let list: Vec<_> = list.iter().map(|s| s.name.as_str()).collect();
                    Menu::from_vec("Select name to delete:", &list).show().to_owned()
                },
                Err(e) => {
                    do_error("Error getting name list", e);
                    std::process::exit(1);
                }
            }
        }
    };

    match db.delete_series(&name) {
        Ok(n) => {
            println!("Deleted {}: {} measurements", name, n);
        },
        Err(e) => {
            do_error("Unable to delete series", e);
            std::process::exit(1);
        }
    };
}
