use clap::{Parser, ValueHint};
use hawk_core::source::csv::CsvIonIterator;
use ion_rs::element::Value;
use std::{fs::File, process};

#[derive(Parser, Debug)]
#[command(name = "hawk")]
#[command(about = "Multi-purpose data utility", version = "0.1")]
struct HawkArgs {
    #[arg(short = 'F')]
    separator: Option<String>,

    #[arg(short = 'q')]
    query: Option<String>,

    #[arg(name = "files", value_hint = ValueHint::FilePath)]
    files: Vec<String>,
}

fn main() {
    let args = HawkArgs::parse();
    println!("{args:?}");

    let csv_file = File::open(&args.files[0]).unwrap();
    let reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(csv_file);
    let query_expr = match hawk_parser::parse_predicate(&args.query.unwrap()) {
        Ok((_, expr)) => expr,
        Err(e) => {
            println!("Error: {:?}", e);
            process::exit(1);
        }
    };
    let ion_iterator = match CsvIonIterator::new(reader) {
        Ok(iter) => iter,
        _ => {
            println!("Could not get iterator");
            process::exit(1);
        }
    };
    for element in ion_iterator {
        if let Some(data) = element.as_struct() {
            let torf = hawk_core::source::resolve_expr(data, &query_expr);
            let torf = torf.unwrap();
            if let Value::Bool(torf) = torf.as_ref() {
                if *torf {
                    println!(">> def: {}", data.get("1").unwrap())
                }
            }
        }
    }
}
