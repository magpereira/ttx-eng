use crate::engine;
use crate::models::tx::TxInput;
use clap::Parser;
use csv::Trim;
use std::error::Error;
use std::io;
use tracing::debug;

/// Simple toy payments engine
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// path of the input file
    pub file_path: String,
}

pub fn process_input<R: io::Read, W: io::Write>(input: R, output: W) -> Result<(), Box<dyn Error>> {
    let mut engine = engine::Engine::new();

    // read from input
    let mut rdr = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_reader(input);

    for result in rdr.deserialize::<TxInput>() {
        let tx = match result {
            Ok(tx) => tx,
            Err(err) => {
                debug!("failed to parse record: {}", err);
                continue
            }
        };

        engine.process_tx(&tx)
    }

    //write to std out
    let mut wtr = csv::Writer::from_writer(output);
    let mut counter = 0;

    for v in engine.report() {
        wtr.serialize(v)?;

        //flush every 1000 lines
        if counter >= 1000 {
            wtr.flush()?;
            counter = 0;
        }

        counter += 1;
    }

    match wtr.flush() {
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
