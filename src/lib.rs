use chrono::{DateTime, NaiveDateTime, Utc};
use exif::{Exif, In, Tag};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Process an image file, ie extract the metadata from it and write out
/// a json file containing that metadata
pub fn process_file<P: AsRef<Path>>(path: P) -> Result<(), ImageError> {
    let metadata = CombinedMetadata {
        file_metadata: file_metadata(&path)?,
        image_metadata: image_metadata(&path)?,
    };
    let mut json_path = path.as_ref().to_path_buf();
    json_path.set_extension("json");
    write_metadata_to_file(json_path, &metadata)?;
    Ok(())
}

/// All the metadata about a file/image
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
struct CombinedMetadata {
    #[serde(flatten)]
    file_metadata: FileMetadata,
    #[serde(flatten)]
    image_metadata: ImageMetadata,
}

/// The file specific metadata
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
struct FileMetadata {
    filename: String,
    size: u64,
    /// Potentially missing if the underlying platform/filesystem doesn't capture the created time
    #[serde(skip_serializing_if = "Option::is_none")]
    created_time: Option<DateTime<Utc>>,
    /// Potentially missing if the underlying platform/filesystem doesn't capture the modified time
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_time: Option<DateTime<Utc>>,
}

/// The metadata from the actual image itself
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
struct ImageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    orientation: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    capture_time: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    camera_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    camera_serial: Option<String>,
}

/// Retrieves the filesystem metadata for a given file.
fn file_metadata<P: AsRef<Path>>(path: P) -> std::io::Result<FileMetadata> {
    let metadata = std::fs::metadata(&path)?;
    Ok(FileMetadata {
        filename: path
            .as_ref()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        size: metadata.len(),
        created_time: metadata.created().map(DateTime::from).ok(),
        modified_time: metadata.modified().map(DateTime::from).ok(),
    })
}

/// Retrieves the filesystem metadata for a given file.
fn image_metadata<P: AsRef<Path>>(path: P) -> Result<ImageMetadata, ImageError> {
    let mut file = std::io::BufReader::new(File::open(path)?);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut file)?;

    /// Attempt to grab the raw bytes and use them as our string to avoid the exif lib
    /// mucking with the raw strings we want out
    fn field_str_unquoted(exif: &Exif, tag: Tag) -> Option<String> {
        exif.get_field(tag, In::PRIMARY).map(|field| {
            if let exif::Value::Ascii(raw) = &field.value {
                let bytes = raw
                    .iter()
                    .flat_map(|i| i.iter())
                    .copied()
                    .collect::<Vec<u8>>();
                String::from_utf8_lossy(&bytes).to_string()
            } else {
                field.display_value().to_string()
            }
        })
    }

    Ok(ImageMetadata {
        orientation: exif
            .get_field(Tag::Orientation, In::PRIMARY)
            .and_then(|f| f.value.get_uint(0)),
        capture_time: field_str_unquoted(&exif, Tag::DateTimeOriginal)
            .and_then(|s| NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S").ok()),
        camera_model: field_str_unquoted(&exif, Tag::Model),
        camera_serial: field_str_unquoted(&exif, Tag::BodySerialNumber),
    })
}

/// Write the metadata out to a file
fn write_metadata_to_file<P: AsRef<Path>>(
    path: P,
    metadata: &CombinedMetadata,
) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    // All these json serde errors will actually be io errors anyway so the conversion
    // back to io wont be lossy.
    serde_json::to_writer_pretty(&mut file, metadata).map_err(std::io::Error::from)?;
    file.flush()?;
    Ok(())
}

/// Top level representation of any error that might happen during image processing
#[derive(Debug)]
pub enum ImageError {
    /// An IO Error and the path that caused it
    IoError(std::io::Error),
    ExifError(exif::Error),
}

impl From<std::io::Error> for ImageError {
    fn from(err: std::io::Error) -> Self {
        ImageError::IoError(err)
    }
}

impl From<exif::Error> for ImageError {
    fn from(err: exif::Error) -> Self {
        ImageError::ExifError(err)
    }
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::IoError(err) => err.fmt(f),
            ImageError::ExifError(err) => err.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};
    use indoc::*;

    #[test]
    fn test_file_metadata() {
        let metadata = file_metadata("tests/images/JAM19896.jpg").unwrap();
        assert_eq!(metadata.filename, "JAM19896.jpg".to_string());
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
    fn test_image_metadata() {
        let metadata = image_metadata("tests/images/JAM19896.jpg").unwrap();
        assert_eq!(metadata.orientation, Some(1));
        assert_eq!(
            metadata.capture_time,
            Some(Local.ymd(2019, 7, 26).and_hms(13, 25, 33).naive_local())
        );
        assert_eq!(
            metadata.camera_model,
            Some("Canon EOS 5D Mark IV".to_string())
        );
        assert_eq!(metadata.camera_serial, Some("025021000537".to_string()));
    }

    #[test]
    fn test_write_metadata_to_file() -> Result<(), ImageError> {
        let file_metadata = FileMetadata {
            filename: "test.jpg".to_string(),
            size: 123456,
            created_time: Some(Utc.ymd(1986, 4, 27).and_hms(7, 30, 0)),
            modified_time: None,
        };
        let image_metadata = ImageMetadata {
            orientation: Some(1),
            capture_time: Some(Local.ymd(1986, 4, 27).and_hms(7, 30, 0).naive_local()),
            camera_model: None,
            camera_serial: Some("baz".to_string()),
        };
        let metadata = CombinedMetadata {
            file_metadata,
            image_metadata,
        };

        let path = "target/test/metadata.json";
        std::fs::create_dir_all("target/test")?;

        write_metadata_to_file(path, &metadata)?;

        assert_eq!(
            std::fs::read_to_string(path)?,
            indoc! {r#"
        {
          "filename": "test.jpg",
          "size": 123456,
          "created_time": "1986-04-27T07:30:00Z",
          "orientation": 1,
          "capture_time": "1986-04-27T07:30:00",
          "camera_serial": "baz"
        }"#}
        );
        Ok(())
    }

    #[test]
    fn test_process_file() -> Result<(), ImageError> {
        // Lets copy our the file under target so we don't pollute the workspace
        std::fs::create_dir_all("target/test")?;
        let image_path = "target/test/JAM19896.jpg";
        let expected_json_path = "target/test/JAM19896.json";
        std::fs::copy("tests/images/JAM19896.jpg", image_path)?;

        process_file(image_path)?;
        let metadata: CombinedMetadata =
            serde_json::from_slice(&std::fs::read(expected_json_path)?).unwrap();
        assert_eq!(metadata.file_metadata.size, 953458);
        Ok(())
    }
}
