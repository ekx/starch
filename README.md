# starch
[![Build Status](https://img.shields.io/github/actions/workflow/status/ekx/starch/.github/workflows/rust.yml)](https://github.com/ekx/starch/actions/workflows/rust.yml)
[![Crates.io version](https://img.shields.io/crates/v/starch.svg)](https://crates.io/crates/starch)
[![GitHub Release](https://img.shields.io/github/v/release/ekx/starch)](https://github.com/ekx/starch/releases)

CLI utility for the Steam version of RetroArch

## Features
Supports Windows and Linux. Should run on macOS but is untested.

- Update all cores (even non Steam cores)
- TODO: Export playlist entries to USB device
- TODO: Import from USB to playlist

## Installation
Preferred installation method is through [cargo](https://www.rust-lang.org/tools/install)
```
cargo install starch
``` 

## Usage
```
starch update-cores
//TODO: starch export 'Sony - PlayStation' 'Tony Hawk's Pro Skater 2 (USA)'
//TODO: starch import 'Nintendo - Super Nintendo Entertainment System' 'Marvelous - Mouhitotsu no Takara-jima (Japan)'
``` 
Detailed usage instruction can be queried with `-h` or `--help`

## Used libraries

* [steamlocate-rs](https://github.com/WilliamVenner/steamlocate-rs) - (MIT)
* [rust-ini](https://github.com/zonyitoo/rust-ini) - (MIT)
* [reqwest](https://github.com/seanmonstar/reqwest) - (MIT / Apache 2.0)
* [tokio](https://github.com/tokio-rs/tokio) - (MIT)
* [sevenz-rust](https://github.com/dyz1990/sevenz-rust) - (Apache 2.0)
* [indicatif](https://github.com/console-rs/indicatif) - (MIT)
* [futures-rs](https://github.com/rust-lang/futures-rs) - (MIT / Apache 2.0)
* [anyhow](https://github.com/dtolnay/anyhow) - (MIT / Apache 2.0)
* [zip](https://github.com/zip-rs/zip) - (MIT)
* [clap](https://github.com/clap-rs/clap) - (MIT / Apache 2.0)

## License
- [MIT](https://github.com/ekx/starch/blob/master/LICENSE)

## Contributing

Please see the [issues section](https://github.com/ekx/starch/issues) to
report any bugs or feature requests and to see the list of known issues.

[Pull requests](https://github.com/ekx/starch/pulls) are always welcome.
