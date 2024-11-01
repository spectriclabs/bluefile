# bluefile

Experimental Rust library for handling X-Midas Bluefiles.

## Usage

Add the following to your project's `Cargo.toml`:

```toml
[dependencies]
bluefile = "*"
```


```rust
use std::fs::File;
use bluefile::read_header;

let file = File::open("/path/to/bluefile").unwrap();
let header = read_header(&file).unwrap();
println!("{}", header.type_code);
println!("{}", header.data_type);
...
```

Additional examples can be found in the `tests` directory and in the `bluejay` utility.

### bluejay

Bluejay is a command line utility for getting bluefile header info in JSON format.

```
bluejay /path/to/bluefile
```

## Running Tests

```sh
cargo clippy
cargo test
```

## Resources
* [Python implementation from RedHawkSDR](https://github.com/RedhawkSDR/framework-core/blob/master/src/base/framework/python/ossie/utils/bluefile/bluefile.py)
* [Javascript implementation from sigfile](https://github.com/LGSInnovations/sigfile/blob/master/src/bluefile.js)
