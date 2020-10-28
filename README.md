# Image-metadata
![Build Status](https://github.com/tim-patterson/image-metadata/workflows/Test/badge.svg)

A command-line utility to extract image metadata from JPEG images

### Building/Running from source
To build and/or run from source you will first need the rustup installed, the install
instructions can be found at https://rustup.rs/

Once this is done you can use the standard cargo commands to build/test/run.

```sh
  # Run (dev build)
  cargo run -- tests/images/*.jpg

  # Run (release build)
  cargo run --release -- tests/images/*.jpg

  # To build an executable run
  cargo build --release

  # To run that executable
  ./target/release/image-metadata tests/images/*.jpg
```

### Testing
To test the code simply run

```sh
  cargo test && cargo clippy
```


### TODO
* [x] File metadata extraction
* [x] Json serialization and output
* [x] Commandline parser and integration test
* [x] Image metadata extraction
* [x] Readme - howto etc
* [x] Friendlier Error messages
