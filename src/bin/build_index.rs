use bincode::serialize_into;
use kseq::parse_path;
use std::env::args;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use suffine::MultiDocIndexBuilder;
use clap::Parser;

use bincode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct TextInfo {
    pub text: String,
    pub target_names: Vec<String>,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
   /// Name of the person to greet
   reference: PathBuf,

   /// Number of times to greet
   index: String,
}

fn main() {

    let args = Args::parse();

    let m_index_base = args.index;//args().nth(2).unwrap();
    let m_text_file = File::create(m_index_base.clone() + ".btex").unwrap();
    let m_index_file = File::create(m_index_base + ".idx").unwrap();
    let mut m_index_writer = BufWriter::new(m_index_file);

    let path: PathBuf = args.reference;//args().nth(1).unwrap();
    let mut records = parse_path(path).unwrap();
    let mut txp_info = TextInfo {
        text: String::new(),
        target_names: vec![],
    };
    //let mut txp_strings = String::new();
    // let mut records = parse_reader(File::open(path).unwrap()).unwrap();
    while let Some(record) = records.iter_record().unwrap() {
        txp_info.text += record.seq();
        txp_info.text.push('#');
        txp_info.target_names.push(record.head().to_string());
    }

    {
        let mut m_text_writer = BufWriter::new(m_text_file);
        serialize_into(&mut m_text_writer, &txp_info).unwrap();
        m_text_writer.flush().unwrap();
    }

    let _multi_doc_index = MultiDocIndexBuilder::new(&txp_info.text)
        .delimiter('#')
        .build_to_writer_native_endian(&mut m_index_writer)
        .unwrap();

    m_index_writer.flush().unwrap();
}
