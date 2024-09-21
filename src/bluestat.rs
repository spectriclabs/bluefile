use std::env;
use std::path::PathBuf;
use std::process::exit;

use bluefile::header::read_header;
use bluefile::error::Error;
use bluefile::result::Result;
use bluefile::util::open_file;

struct Config {
    path: PathBuf,
}

fn get_config() -> Result<Config> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Configuration error");
        return Err(Error::BluestatConfigError);
    }

    let path_str = args[1].trim();

    if path_str.len() == 0 {
        println!("Bluefile path is empty string");
        return Err(Error::BluestatConfigError);
    }

    let mut path_buf = PathBuf::new();
    path_buf.push(path_str);

    Ok(Config{path: path_buf})
}

fn main() {
    let config = match get_config() {
        Ok(c) => c,
        Err(_) => exit(1),
    };


    let file = match open_file(&config.path) {
        Ok(f) => f,
        Err(_) => {
            println!("Could not open file at {}", config.path.display());
            exit(1);
        },
    };

    let header = match read_header(&file) {
        Ok(h) => h,
        Err(_) => {
            println!("Could not read header from {}", config.path.display());
            exit(1);
        },
    };

    dbg!(header);
}
