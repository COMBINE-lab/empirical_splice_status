use anyhow::Result;
use bincode;
use kseq::parse_path;
use memmap::Mmap;
use serde::{Deserialize, Serialize};
use std::env::args;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use suffine::MultiDocIndex;

fn open_and_map<P: AsRef<Path>>(path: P) -> Result<Mmap> {
    let file = File::open(&path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    Ok(mmap)
}

#[derive(Serialize, Deserialize, Debug)]
struct TextInfo {
    pub text: String,
    pub target_names: Vec<String>,
}

enum SplicingStatus {
    MATURE,
    NASCENT,
    AMBIGUOUS,
}

impl fmt::Display for SplicingStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SplicingStatus::MATURE => write!(f, "m"),
            SplicingStatus::NASCENT => write!(f, "n"),
            SplicingStatus::AMBIGUOUS => write!(f, "a"),
        }
    }
}

fn revcomp(dna: &str) -> String {
    // result vector
    let mut rdna: String = String::with_capacity(dna.len());

    // iterate through the input &str
    for c in dna.chars().rev() {
        // test the input
        match is_dna(c) {
            false => panic!("Input sequence base is not DNA: {}", dna),
            true => rdna.push(switch_base(c)),
        }
    }
    rdna
}

fn switch_base(c: char) -> char {
    match c {
        'a' => 't',
        'c' => 'g',
        't' => 'a',
        'g' => 'c',
        'u' => 'a',
        'A' => 'T',
        'C' => 'G',
        'T' => 'A',
        'G' => 'C',
        'U' => 'A',
        _ => 'N',
    }
}

fn is_dna(dna: char) -> bool {
    match dna {
        'A' | 'a' | 'C' | 'c' | 'G' | 'g' | 'T' | 't' | 'U' | 'u' => true,
        _ => false,
    }
}

fn main() -> Result<(), anyhow::Error> {
    let base_filename = args().nth(1).unwrap();
    let text_filename = base_filename.clone() + ".btex";
    let idx_filename = base_filename.clone() + ".idx";

    let text_file = File::open(text_filename).unwrap();
    let mut reader = BufReader::new(text_file);
    let txp_info: TextInfo = bincode::deserialize_from(&mut reader)?;

    let m_index_mmap = open_and_map(idx_filename)?;
    let multi_doc_index = MultiDocIndex::from_bytes(&txp_info.text, &m_index_mmap)?;

    let path: String = args().nth(2).unwrap();
    let mut records = parse_path(path).unwrap();

    use std::io::Write;
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    // let mut records = parse_reader(File::open(path).unwrap()).unwrap();
    while let Some(record) = records.iter_record().unwrap() {
        let query = record.seq();
        let query_rc = revcomp(query);
        let mut had_nascent = false;
        let mut had_mature = false;
        let mut status: Option<SplicingStatus> = None;
        for q in [query, &query_rc] {
            for (doc_id, _pos) in multi_doc_index.doc_positions(q) {
                let doc_name = &txp_info.target_names[doc_id as usize];
                if doc_name.ends_with("-I") {
                    had_nascent = true;
                    status = Some(SplicingStatus::NASCENT);
                } else {
                    had_mature = true;
                    status = Some(SplicingStatus::MATURE);
                }
                if had_nascent && had_mature {
                    status = Some(SplicingStatus::AMBIGUOUS);
                    break;
                }
            }
            if let Some(s) = status {
                writeln!(lock, "{}\t{}", record.head(), s)?;
                break;
            }
        }
    }
    Ok(())
}
