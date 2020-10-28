use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Process an image file, ie extract the metadata from it and write out
/// a json file containing that metadata
pub fn process_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let metadata = file_metadata(&path)?;
    let mut json_path = path.as_ref().to_path_buf();
    json_path.set_extension("json");
    write_metadata_to_file(json_path, &metadata)?;
    Ok(())
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
struct ImageMetadata {
    size: u64,
    /// Potentially missing if the underlying platform/filesystem doesn't capture the created time
    created_time: Option<DateTime<Utc>>,
    /// Potentially missing if the underlying platform/filesystem doesn't capture the modified time
    modified_time: Option<DateTime<Utc>>,
}

/// Retrieves the filesystem metadata for a given file.
fn file_metadata<P: AsRef<Path>>(path: P) -> std::io::Result<ImageMetadata> {
    let metadata = std::fs::metadata(path)?;
    Ok(ImageMetadata {
        size: metadata.len(),
        created_time: metadata.created().map(DateTime::from).ok(),
        modified_time: metadata.modified().map(DateTime::from).ok(),
    })
}

/// Write the metadata out to a file
fn write_metadata_to_file<P: AsRef<Path>>(
    path: P,
    metadata: &ImageMetadata,
) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    // All these json serde errors will actually be io errors anyway so the conversion
    // back to io wont be lossy.
    serde_json::to_writer_pretty(&mut file, metadata)?;
    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use indoc::*;

    #[test]
    fn test_file_metadata() {
        let metadata = file_metadata("tests/images/JAM19896.jpg").unwrap();
        assert_eq!(metadata.size, 953458);
        // Git wont restore ctime/crtime so the best we can do is see if we get something sane
        let created_time = metadata.created_time.expect("Created time expected");
        let modified_time = metadata.created_time.expect("Modified time expected");
        // A system time representing the start of 2020
        let t_2020 = Utc.ymd(2020, 1, 1).and_hms(0, 0, 0);
        let now = Utc::now();
        assert!(created_time > t_2020);
        assert!(created_time <= now);
        assert!(modified_time > t_2020);
        assert!(modified_time <= now);
    }

    #[test]
    fn test_write_metadata_to_file() -> std::io::Result<()> {
        let metadata = ImageMetadata {
            size: 123456,
            created_time: Some(Utc.ymd(1986, 4, 27).and_hms(7, 30, 0)),
            modified_time: None,
        };
        let path = "target/test/metadata.json";
        std::fs::create_dir_all("target/test")?;

        write_metadata_to_file(path, &metadata)?;

        assert_eq!(
            std::fs::read(path)?,
            indoc! {br#"
        {
          "size": 123456,
          "created_time": "1986-04-27T07:30:00Z",
          "modified_time": null
        }"#}
        );
        Ok(())
    }

    #[test]
    fn test_process_file() -> std::io::Result<()> {
        // Lets copy our the file under target so we don't pollute the workspace
        std::fs::create_dir_all("target/test")?;
        let image_path = "target/test/JAM19896.jpg";
        let expected_json_path = "target/test/JAM19896.json";
        std::fs::copy("tests/images/JAM19896.jpg", image_path)?;

        process_file(image_path)?;
        let metadata: ImageMetadata = serde_json::from_slice(&std::fs::read(expected_json_path)?)?;
        assert_eq!(metadata.size, 953458);
        Ok(())
    }
}
