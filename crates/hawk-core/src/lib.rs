use anyhow::Result;
use ion_rs::element::reader::ElementReader;
use ion_rs::ReaderBuilder;
use std::fs::File;

pub mod source;

pub fn read_some_ion_data() -> Result<()> {
    let ion_file = File::open("data.ion").unwrap();
    let mut reader = ReaderBuilder::default().build(ion_file)?;
    // A simple pretty-printer
    for element in reader.elements() {
        // Check if the `Element` is a struct
        if let Some(ion_struct) = element?.as_struct() {
            if let Some(def) = ion_struct.get("def") {
                println!(">> def: {}", def);
            }
        }
    }
    Ok(())
}

pub fn read_some_csv_data() -> Result<()> {
    let csv_file = File::open("data.csv").unwrap();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(csv_file);
    for result in rdr.records() {
        let record = result?;
        println!("field 3: {:?}", record.get(2));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_some_ion_data() {
        let result = read_some_ion_data();
        assert!(result.is_ok(), "Expected Ok but got Err: {:?}", result);
    }

    #[test]
    fn test_read_some_csv_data() {
        let result = read_some_csv_data();
        assert!(result.is_ok(), "Expected Ok but got Err: {:?}", result);
    }
}
