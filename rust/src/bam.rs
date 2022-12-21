use crate::run::depth;
use crate::utils::styled_progress_bar;
use rust_htslib::bam::{index, Header, IndexedReader, Read};
use rust_htslib::htslib;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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
) -> IndexedReader {
    let bam_cram_path = match bam_path {
        None => cram_path.as_ref().unwrap(),
        &Some(_) => bam_path.as_ref().unwrap(),
    };
    create_index(&bam_cram_path);
    let bam = IndexedReader::from_path(&bam_cram_path).unwrap();
    bam
}

pub fn reads_from_bam(seq_names: &HashSet<Vec<u8>>, mut bam: IndexedReader) -> HashSet<Vec<u8>> {
    let mut wanted_reads = HashSet::new();
    let total = seq_names.len() as u64;
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
            // TODO: add option to print info from matching records file, e.g.
            // println!(
            //     "{:?}: {:?}",
            //     String::from_utf8(read.qname().to_vec()).unwrap(),
            //     read.cigar().to_string()
            // );
        }
        progress_bar.inc(1);
    }
    progress_bar.finish();
    wanted_reads
}

fn seq_lengths_from_header(
    bam: &IndexedReader,
    seq_names: &HashSet<Vec<u8>>,
) -> HashMap<String, u64> {
    let header = Header::from_template(bam.header());
    let mut seq_lengths: HashMap<String, u64> = HashMap::new();
    for (_, records) in header.to_hashmap() {
        for record in records {
            if record.contains_key("SN") {
                if seq_names.len() > 0 && !seq_names.contains(&record["SN"].as_bytes().to_vec()) {
                    continue;
                }
                seq_lengths
                    .entry(record["SN"].to_string())
                    .or_insert(record["LN"].parse::<u64>().unwrap());
            }
        }
    }
    seq_lengths
}

pub fn depth_from_bam(
    seq_lengths: &HashMap<String, u64>,
    mut bam: IndexedReader,
    bin_size: &u32,
) -> () {
    let total = seq_lengths.len() as u64;
    let progress_bar = styled_progress_bar(total, "Locating alignments");
    let step = *bin_size as usize;
    for (seq_name, length) in seq_lengths {
        match bam.fetch(seq_name) {
            Err(_) => eprintln!("Sequence {:?} not found in BAM file", seq_name),
            Ok(_) => (),
        }
        let mut binned_cov: Vec<u64> = vec![];
        for _ in (0..*length).step_by(step) {
            binned_cov.push(0)
        }

        for p in bam.pileup() {
            let pileup = p.unwrap();
            let bin = pileup.pos() as usize / step;
            binned_cov[bin] += 1;
        }

        println!("{:?}", binned_cov);
        progress_bar.inc(1);
        break;
    }
    progress_bar.finish();
}

pub fn get_depth(bam: IndexedReader, seq_names: &HashSet<Vec<u8>>, bin_size: &u32) -> () {
    let seq_lengths = seq_lengths_from_header(&bam, &seq_names);
    depth_from_bam(&seq_lengths, bam, bin_size);
    // let mut seq_names = names_list;
    // let alt_names: HashSet<Vec<u8>>;
    // if names_list.len() == 0 {
    //     alt_names = seq_names_from_header(&bam);
    //     seq_names = &alt_names;
    // }
    println!("{:?}", seq_lengths);
}
