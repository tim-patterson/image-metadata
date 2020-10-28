use assert_cmd::Command;
use serde_json::Value;

/// A simple top level test to check the cli
#[test]
fn test_cli_happy() {
    // Lets copy our the file under target so we don't pollute the workspace
    std::fs::create_dir_all("target/test").expect("Failed to create directory");
    let image_path_1 = "target/test/JAM19896.jpg";
    let expected_json_path_1 = "target/test/JAM19896.json";
    let image_path_2 = "target/test/JAM26284.jpg";
    let expected_json_path_2 = "target/test/JAM26284.json";
    std::fs::copy("tests/images/JAM19896.jpg", image_path_1).expect("Failed to copy test file");
    std::fs::copy("tests/images/JAM26284.jpg", image_path_2).expect("Failed to copy test file");

    let mut cmd = Command::cargo_bin("image-metadata").unwrap();
    cmd.arg(image_path_1).arg(image_path_2).assert().success();

    let metadata: Value =
        serde_json::from_slice(&std::fs::read(expected_json_path_1).unwrap()).unwrap();
    assert_eq!(metadata.get("size").unwrap().as_u64(), Some(953458));

    let metadata: Value =
        serde_json::from_slice(&std::fs::read(expected_json_path_2).unwrap()).unwrap();
    assert_eq!(metadata.get("size").unwrap().as_u64(), Some(574207));
}

/// A simple top level test to check the returns an error code/message
#[test]
fn test_cli_sad() {
    std::fs::create_dir_all("target/test").expect("Failed to create directory");

    let mut cmd = Command::cargo_bin("image-metadata").unwrap();
    cmd.arg("file_that_doesnt_exist").assert().failure().stderr(
        r#"Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }
"#,
    );
}