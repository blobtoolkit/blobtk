extern crate atty;
use std::collections::HashSet;
use std::io::{self, BufRead, BufWriter, Result, Write};
use std::path::{Path, PathBuf};

use std::fs::File;

use flate2::write;
use flate2::Compression;
use std::ffi::OsStr;

fn read_stdin() -> Vec<Vec<u8>> {
    let stdin = io::stdin();
    let mut list: Vec<Vec<u8>> = vec![];
    if atty::is(atty::Stream::Stdin) {
        eprintln!("No input on STDIN!");
        return list;
    }
    for line in stdin.lock().lines() {
        let line_as_vec = match line {
            Err(why) => panic!("couldn't read line: {}", why),
            Ok(l) => l.as_bytes().to_vec(),
        };
        list.push(line_as_vec)
    }
    list
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).expect("no such file");
    Ok(io::BufReader::new(file).lines())
}

fn read_file(file_path: &PathBuf) -> Vec<Vec<u8>> {
    let mut output: Vec<Vec<u8>> = vec![];
    if let Ok(lines) = read_lines(file_path) {
        for line in lines {
            let line_as_vec = match line {
                Err(why) => panic!("couldn't read line: {}", why),
                Ok(l) => l.as_bytes().to_vec(),
            };
            output.push(line_as_vec)
        }
    }
    output
}

pub fn get_list(file_path: &Option<PathBuf>) -> HashSet<Vec<u8>> {
    let list = match file_path {
        None => vec![],
        Some(p) if p == Path::new("-") => read_stdin(),
        Some(_) => read_file(file_path.as_ref().unwrap()),
    };
    HashSet::from_iter(list)
}

pub fn get_file_writer(file_path: &PathBuf) -> Box<dyn Write> {
    let file = match File::create(file_path) {
        Err(why) => panic!("couldn't open {}: {}", file_path.display(), why),
        Ok(file) => file,
    };

    let writer: Box<dyn Write> = if file_path.extension() == Some(OsStr::new("gz")) {
        Box::new(BufWriter::with_capacity(
            128 * 1024,
            write::GzEncoder::new(file, Compression::default()),
        ))
    } else {
        Box::new(BufWriter::with_capacity(128 * 1024, file))
    };
    writer
}

pub fn get_writer(file_path: &Option<PathBuf>) -> Box<dyn Write> {
    let writer: Box<dyn Write> = match file_path {
        Some(path) if path == Path::new("-") => Box::new(BufWriter::new(io::stdout().lock())),
        Some(path) => get_file_writer(path),
        None => Box::new(BufWriter::new(io::stdout().lock())),
    };
    writer
}

pub fn write_list(entries: &HashSet<Vec<u8>>, file_path: &Option<PathBuf>) -> Result<()> {
    let mut writer = get_writer(file_path);
    for line in entries.iter() {
        writeln!(&mut writer, "{}", String::from_utf8(line.to_vec()).unwrap())?;
    }
    Ok(())
}
