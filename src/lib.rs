use std::path::Path;
use std::time::SystemTime;

/// Image metadata
#[derive(Eq, PartialEq, Debug)]
pub struct ImageMetadata {
    size: u64,
    /// Potentially missing if the underlying platform/filesystem doesn't capture the created time
    created_time: Option<SystemTime>,
    /// Potentially missing if the underlying platform/filesystem doesn't capture the modified time
    modified_time: Option<SystemTime>,
}

/// Retrieves the filesystem metadata for a given file.
pub fn file_metadata<P: AsRef<Path>>(path: P) -> std::io::Result<ImageMetadata> {
    let metadata = std::fs::metadata(path)?;
    Ok(ImageMetadata {
        size: metadata.len(),
        created_time: metadata.created().ok(),
        modified_time: metadata.modified().ok(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;
    use std::ops::Add;

    #[test]
    fn test_file_metadata() {
        let metadata = file_metadata("tests/images/JAM19896.jpg").unwrap();
        assert_eq!(metadata.size, 953458);
        // Git wont restore ctime/crtime so the best we can do is see if we get something sane
        let created_time = metadata.created_time.expect("Created time expected");
        let modified_time = metadata.created_time.expect("Modified time expected");
        // A system time representing the start of 2020
        let t_2020 = SystemTime::UNIX_EPOCH.add(Duration::from_secs(1575072000));
        let now = SystemTime::now();
        assert!(created_time > t_2020);
        assert!(created_time <= now);
        assert!(modified_time > t_2020);
        assert!(modified_time <= now);
    }
}
