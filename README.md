# bluefile

Experimental Rust library for handling X-Midas Bluefiles.

## Usage

Add the following to your project's `Cargo.toml`:

```toml
[dependencies]
bluefile = "*"
```

### Reading Type 2000 frames

```rust
use std::path::PathBuf;

use bluefile::bluefile::BluefileReader;
use bluefile::data_type::DataValue;
use bluefile::type2000::Type2000Reader;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = PathBuf::from(&args[1]);
    let reader = Type2000Reader::new(&path).unwrap();
    let adj_header = &reader.get_adj_header();
    let frame_size: usize = adj_header.subsize.try_into().unwrap();
    let data_reader = &mut reader.get_data_iter().unwrap();

    loop {
        let frame: Vec<DataValue> = data_reader.take(frame_size).collect();

        if frame.len() < frame_size {
            break;
        }

        dbg!(frame);
    }
}
```

More examples can be found in the `tests` directory.

## Running Tests

```sh
cargo clippy
cargo test
```

## Resources
* [Python implementation from RedHawkSDR](https://github.com/RedhawkSDR/framework-core/blob/master/src/base/framework/python/ossie/utils/bluefile/bluefile.py)
* [Javascript implementation from sigfile](https://github.com/LGSInnovations/sigfile/blob/master/src/bluefile.js)
