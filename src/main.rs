use clap::{App, Arg};
use image_metadata::process_file;

fn main() -> std::io::Result<()> {
    let matches = App::new("Image Metadata Extractor")
        .about("Extracts metadata from image files into json")
        .arg(Arg::with_name("FILES").required(true).multiple(true))
        .get_matches();

    for path in matches.values_of_os("FILES").unwrap() {
        process_file(path)?;
    }
    Ok(())
}
