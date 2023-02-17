use std::collections::HashSet;
use std::io::{ErrorKind, Result, Write};
// use std::ops::Index;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use rust_htslib::bam::{index, Header, IndexedReader, Read};
use rust_htslib::htslib;

use crate::cli::DepthOptions;
use crate::io::get_writer;
use crate::utils::styled_progress_bar;

fn add_extension(path: &mut PathBuf, extension: impl AsRef<Path>) {
    match path.extension() {
        Some(ext) => {
            let mut ext = ext.to_os_string();
            ext.push(".");
            ext.push(extension.as_ref());
            path.set_extension(ext)
        }
        None => path.set_extension(extension.as_ref()),
    };
}

pub fn create_index(bam_path: &PathBuf) -> () {
    let mut csi = PathBuf::from(bam_path);
    add_extension(&mut csi, "csi");
    if Path::new(&csi).exists() {
        return;
    }
    let mut bai = PathBuf::from(bam_path);
    add_extension(&mut bai, "bai");
    if Path::new(&bai).exists() {
        return;
    }
    match index::build(bam_path, None, index::Type::Csi(14), 1) {
        Err(e) => eprintln!("Error writing BAM index: {e:?}"),
        Ok(_) => eprintln!("Successfully created BAM index"),
    }
}

pub fn open_bam(
    bam_path: &Option<PathBuf>,
    cram_path: &Option<PathBuf>,
    _fasta_path: &Option<PathBuf>,
    make_index: bool,
) -> IndexedReader {
    let bam_cram_path = match bam_path {
        None => cram_path.as_ref().unwrap(),
        &Some(_) => bam_path.as_ref().unwrap(),
    };
    if make_index {
        create_index(&bam_cram_path);
    }
    let bam = IndexedReader::from_path(&bam_cram_path).unwrap();
    bam
}

pub fn reads_from_bam(seq_names: &HashSet<Vec<u8>>, mut bam: IndexedReader) -> HashSet<Vec<u8>> {
    let mut wanted_reads = HashSet::new();
    let total = seq_names.len();
    let progress_bar = styled_progress_bar(total, "Locating alignments");

    for seq_name in seq_names {
        match bam.fetch(seq_name) {
            Err(_) => eprintln!("Sequence {:?} not found in BAM file", seq_name),
            Ok(_) => (),
        }

        for read in bam
            .rc_records()
            .map(|x| x.expect("Failure parsing Bam file"))
            // TODO: include filter options in config
            .filter(|read| {
                read.flags()
                    & (htslib::BAM_FUNMAP
                        | htslib::BAM_FSECONDARY
                        | htslib::BAM_FQCFAIL
                        | htslib::BAM_FDUP) as u16
                    == 0
            })
        {
            wanted_reads.insert(read.qname().to_vec());
        }
        progress_bar.inc(1);
    }
    progress_bar.finish();
    wanted_reads
}

fn seq_lengths_from_header(
    bam: &IndexedReader,
    seq_names: &HashSet<Vec<u8>>,
) -> IndexMap<String, usize> {
    let header = Header::from_template(bam.header());
    let mut seq_lengths: IndexMap<String, usize> = IndexMap::new();
    for (_, records) in header.to_hashmap() {
        for record in records {
            if record.contains_key("SN") {
                if seq_names.len() > 0 && !seq_names.contains(&record["SN"].as_bytes().to_vec()) {
                    continue;
                }
                seq_lengths
                    .entry(record["SN"].to_string())
                    .or_insert(record["LN"].parse::<usize>().unwrap());
            }
        }
    }
    seq_lengths
}

// #[derive(Clone, Debug)]
// pub struct BinnedCov {
//     seq_name: String,
//     bins: Vec<f64>,
//     bin_count: usize,
//     last_bin: usize,
//     seq_length: usize,
//     step: usize,
// }

// impl BinnedCov {
//     pub fn seq_name(self) -> String {
//         self.seq_name
//     }
//     pub fn bins(self) -> Vec<f64> {
//         self.bins
//     }
//     pub fn bin_count(self) -> usize {
//         self.bin_count
//     }
//     pub fn seq_length(self) -> usize {
//         self.seq_length
//     }
//     pub fn last_bin(self) -> usize {
//         self.last_bin
//     }
//     pub fn step(self) -> usize {
//         self.step
//     }
// }

fn depth_to_bed(
    raw_cov: Vec<usize>,
    length: &usize,
    step: usize,
    seq_name: &String,
    writer: &mut Box<dyn Write>,
) -> Result<()> {
    let mut bins: Vec<f64> = vec![];
    let mut divisor = step;
    let mut end: usize = 0;
    let seq_length = length.to_owned();
    for cov in raw_cov {
        end += step;
        if end > seq_length {
            divisor -= end - seq_length;
        }
        bins.push(cov as f64 / divisor as f64);
    }
    // let binned_cov = BinnedCov {
    //     seq_name: seq_name.to_owned(),
    //     step,
    //     bin_count: bins.len(),
    //     bins,
    //     seq_length,
    //     last_bin: divisor,
    // };
    let mut start = 0;
    let mut end;
    let bin_count = bins.len();
    for i in 0..bin_count {
        end = start + step;
        if end > seq_length {
            end = seq_length;
        }
        let line = format!("{}\t{}\t{}\t{:.2}", seq_name, start, end, bins[i]);
        match writeln!(writer, "{}", line) {
            Err(err) => return Err(err),
            Ok(_) => (),
        };
        start = end;
    }
    Ok(())
}

pub fn depth_from_bam(
    seq_lengths: &IndexMap<String, usize>,
    mut bam: IndexedReader,
    options: &DepthOptions,
) -> () {
    let total = seq_lengths.len();
    let progress_bar = styled_progress_bar(total, "Locating alignments");
    let bin_size = options.bin_size;
    let step = bin_size;
    println!("{:?}", &options.output);
    let mut writer = get_writer(&options.output);
    for (seq_name, length) in seq_lengths.clone() {
        let mut raw_cov: Vec<usize> = vec![];
        for _ in (0..length).step_by(step) {
            raw_cov.push(0)
        }
        match bam.fetch(&seq_name) {
            Err(_) => eprintln!("Sequence {:?} not found in BAM file", seq_name),
            Ok(_) => (),
        }
        for p in bam.pileup() {
            let pileup = p.unwrap();
            let bin = pileup.pos() as usize / step;
            raw_cov[bin] += pileup.depth() as usize;
        }
        match depth_to_bed(raw_cov, &length, step, &seq_name, &mut writer) {
            Err(err) if err.kind() == ErrorKind::BrokenPipe => return,
            Err(err) => panic!("unable to write {} to bed file: {}", &seq_name, err),
            Ok(_) => (),
        };
        progress_bar.inc(1);
    }
    progress_bar.finish();
}

pub fn get_depth(bam: IndexedReader, seq_names: &HashSet<Vec<u8>>, options: &DepthOptions) -> () {
    let seq_lengths = seq_lengths_from_header(&bam, &seq_names);
    depth_from_bam(&seq_lengths, bam, options);
}
