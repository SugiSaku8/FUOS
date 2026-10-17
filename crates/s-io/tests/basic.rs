use std::fs;

use s_io::{Metadata, SioReader, SioWriter};

#[test]
fn test_basic_write_read() {
    let path = "test_basic.sio";

    //
    // Write
    //

    let mut writer = SioWriter::create(path).unwrap();

    let metadata = Metadata {
        name: "Test Ocean Dataset".to_string(),

        version: "0.1.0".to_string(),

        description: "s-io test data".to_string(),
    };

    writer.write_metadata(&metadata).unwrap();

    writer
        .write_array("velocity_u", &[1.0, 2.0, 3.0, 4.0])
        .unwrap();

    writer
        .write_array("temperature", &[15.0, 16.0, 17.0])
        .unwrap();

    writer.finish().unwrap();

    //
    // Read
    //

    let mut reader = SioReader::open(path).unwrap();

    assert_eq!(reader.metadata().name, "Test Ocean Dataset");

    assert_eq!(reader.datasets().len(), 2);

    let velocity = reader.read_dataset("velocity_u").unwrap();

    assert_eq!(velocity, vec![1.0, 2.0, 3.0, 4.0]);

    let temperature = reader.read_dataset("temperature").unwrap();

    assert_eq!(temperature, vec![15.0, 16.0, 17.0]);

    //
    // Cleanup
    //

    fs::remove_file(path).unwrap();
}
