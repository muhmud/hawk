use anyhow::Result;
use std::io::Read;

use csv::StringRecord;
use ion_rs::element::Element;

pub struct CsvIonIterator<R: Read> {
    reader: csv::Reader<R>,
    headers: Option<StringRecord>,
}

impl<R: Read> CsvIonIterator<R> {
    pub fn new(mut reader: csv::Reader<R>) -> Result<Self> {
        let headers = if reader.has_headers() {
            Some(reader.headers()?.to_owned())
        } else {
            None
        };
        Ok(CsvIonIterator::<R> { reader, headers })
    }

    fn get_header(&self, index: usize) -> Option<&str> {
        let headers = self.headers.as_ref();
        if let Some(headers) = headers {
            headers.get(index)
        } else {
            None
        }
    }
}

impl<R: Read> Iterator for CsvIonIterator<R> {
    type Item = Element;

    fn next(&mut self) -> Option<Element> {
        match self.reader.records().next() {
            Some(Ok(record)) => {
                let mut builder = Element::struct_builder();
                for (i, field) in record.iter().enumerate() {
                    let default_name = format!("{i}");
                    let name = self.get_header(i).unwrap_or(&default_name);
                    builder = builder.with_field(name, field);
                }
                Some(builder.build().into())
            }
            _ => None,
        }
    }
}
