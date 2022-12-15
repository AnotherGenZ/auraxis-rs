# auraxis-rs
This project aims to provide an easy way to get started developing for Daybreak Game's Census API in Rust, currently targeted for Planetside 2.

## Getting started
The cargo docs are provided on [github pages](https://anothergenz.github.io/auraxis-rs/auraxis/).
To get started with this crate you can take a look at the [examples](auraxis/examples/) folder for the auraxis crate. For now this crate is not published to crates.io and can be added to cargo.toml like follows:
```toml
[dependencies]
auraxis = { git = "https://github.com/AnotherGenZ/auraxis-rs", version = "0.1.0" }
```
If you want to use a local development version of the crate you can use the following in addition to the two lines above:
```toml
[patch."https://github.com/AnotherGenZ/auraxis-rs"]
auraxis = { path = "/path/to/auraxis-rs/auraxis" }
```

