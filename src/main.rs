use clap::{App, Arg};
use image_metadata::process_file;
use std::process::exit;

fn main() {
    let matches = App::new("Image Metadata Extractor")
        .about("Extracts metadata from image files into json")
        .arg(Arg::with_name("FILES").required(true).multiple(true))
        .get_matches();

    for path in matches.values_of_os("FILES").unwrap() {
        if let Err(error) = process_file(path) {
            eprintln!(
                "While processing {}, we hit error {}",
                path.to_string_lossy(),
                error
            );
            exit(1)
        }
    }
}
