use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;

use bluefile::{
    Error,
    Header,
    read_ext_header,
    read_header,
    read_type1000_adjunct_header,
    read_type2000_adjunct_header,
    Result,
};

struct Config {
    file: File,
    path: PathBuf,
}

fn get_config() -> Result<Config> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Configuration error");
        return Err(Error::BluejayConfigError);
    }

    let path_str = args[1].trim();

    if path_str.is_empty() {
        println!("Bluefile path is empty string");
        return Err(Error::BluejayConfigError);
    }

    let mut path_buf = PathBuf::new();
    path_buf.push(path_str);

    let file = match File::open(&path_buf) {
        Ok(x) => x,
        Err(_) => return Err(Error::FileOpenError(path_buf.display().to_string())),
    };

    Ok(Config{
        file,
        path: path_buf,
    })
}

fn header_lines(header: &Header, lines: &mut Vec<String>) {
    lines.push(format!("  \"type_code\": \"{}\",", header.type_code));
    lines.push(format!("  \"header_endianness\": \"{}\",", header.header_endianness));
    lines.push(format!("  \"data_endianness\": \"{}\",", header.data_endianness));
    lines.push(format!("  \"ext_header_start\": {},", header.ext_start));
    lines.push(format!("  \"ext_header_size\": {},", header.ext_size));
    lines.push(format!("  \"data_start\": {},", header.data_start));
    lines.push(format!("  \"data_size\": {},", header.data_size));
    lines.push(format!("  \"data_type\": \"{}\",", header.data_type));
    lines.push(format!("  \"timecode\": {},", header.timecode));
}

fn adjunct_lines(file: &File, header: &Header, lines: &mut Vec<String>) {
    match header.type_code / 1000 {
        1 => {
            let adj = match read_type1000_adjunct_header(file, header) {
                Ok(a) => a,
                Err(_) => {
                    println!("Error reading adjunct header");
                    return;
                }
            };

            lines.push(format!("  \"xstart\": {},", adj.xstart));
            lines.push(format!("  \"xdelta\": {},", adj.xdelta));
            lines.push(format!("  \"xunits\": {},", adj.xunits));
        },
        2 => {
            let adj = match read_type2000_adjunct_header(file, header) {
                Ok(a) => a,
                Err(_) => {
                    println!("Error reading adjunct header");
                    return;
                }
            };

            lines.push(format!("  \"xstart\": {},", adj.xstart));
            lines.push(format!("  \"xdelta\": {},", adj.xdelta));
            lines.push(format!("  \"xunits\": {},", adj.xunits));
            lines.push(format!("  \"subsize\": {},", adj.subsize));
            lines.push(format!("  \"ystart\": {},", adj.ystart));
            lines.push(format!("  \"ydelta\": {},", adj.ydelta));
            lines.push(format!("  \"yunits\": {},", adj.yunits));
        },
        _ => {},
    }
}

fn keyword_lines(header: &Header, lines: &mut Vec<String>) {
    if header.keywords.len() == 0 {
        lines.push("  \"keywords\": [],".to_string());
        return;
    }

    lines.push("  \"keywords\": [".to_string());
    let last_index = header.keywords.len() - 1;

    for i in 0..header.keywords.len() {
        let keyword = &header.keywords[i];

        if i == last_index {
            lines.push(format!("    {{ \"name\": \"{}\", \"value\": \"{}\" }}", keyword.name, keyword.value));
        } else {
            lines.push(format!("    {{ \"name\": \"{}\", \"value\": \"{}\" }},", keyword.name, keyword.value));
        }
    }

    lines.push("  ],".to_string());
}

fn ext_header_lines(file: &File, header: &Header, lines: &mut Vec<String>) {
    let keywords = match read_ext_header(file, header) {
        Ok(x) => x,
        Err(_) => {
            println!("Could not read extended header");
            exit(1);
        },
    };

    if keywords.len() == 0 {
        lines.push("  \"ext_header\": []".to_string());
        return;
    }

    lines.push("  \"ext_header\": [".to_string());
    let last_index = keywords.len() - 1;

    for i in 0..keywords.len() {
        let keyword = &keywords[i];

        if i == last_index {
            lines.push(format!("    {{ \"name\": \"{}\", \"value\": {}, \"format\": \"{}\" }}", keyword.tag, keyword.value, keyword.value.format));
        } else {
            lines.push(format!("    {{ \"name\": \"{}\", \"value\": {}, \"format\": \"{}\" }},", keyword.tag, keyword.value, keyword.value.format));
        }
    }

    lines.push("  ]".to_string());
}

fn main() {
    let config = match get_config() {
        Ok(c) => c,
        Err(_) => exit(1),
    };

    let header = match read_header(&config.file) {
        Ok(h) => h,
        Err(_) => {
            println!("Could not read header from {}", config.path.display());
            exit(1);
        },
    };

    let mut lines: Vec<String> = vec![];
    header_lines(&header, &mut lines);
    adjunct_lines(&config.file, &header, &mut lines);
    keyword_lines(&header, &mut lines);
    ext_header_lines(&config.file, &header, &mut lines);
    let all_lines = lines.join("\n");

    println!("{{");
    println!("{}", all_lines);
    println!("}}");
}
