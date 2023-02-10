use clap::{ArgGroup, Parser, Subcommand};
use std::path::PathBuf;

fn float_range(s: &str, min: f64, max: f64) -> Result<f64, String> {
    debug_assert!(min <= max, "minimum of {} exceeds maximum of {}", min, max);
    let val = match s.parse::<f64>() {
        Ok(v) => v,
        Err(e) => panic!("{:?}", e),
    };
    check_float_range(val, min, max)
}

fn check_float_range(val: f64, min: f64, max: f64) -> Result<f64, String> {
    if val > max {
        Err(format!("exceeds maximum of {}", max))
    } else if val < min {
        Err(format!("less than minimum of {}", min))
    } else {
        Ok(val)
    }
}

fn window_size_range(s: &str) -> Result<f64, String> {
    float_range(s, 0.0001, 1000000000.0)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// Filter files based on list of sequence names
    Filter(FilterOptions),
    /// Calculate read/base coverage depth
    Depth(DepthOptions),
}

#[derive(Parser, Debug)]
#[command(group(
    ArgGroup::new("alignment")
        .required(false)
        .args(["bam", "cram"]),
))]
pub struct FilterOptions {
    /// Path to input file containing a list of sequence IDs
    // TODO: add option to invert list (use BAM header)
    #[arg(long = "list", short = 'i', value_name = "TXT")]
    pub list_file: Option<PathBuf>,
    /// Path to BAM file
    #[arg(long, short = 'b')]
    pub bam: Option<PathBuf>,
    /// Path to CRAM file
    #[arg(long, short = 'c', requires = "fasta")]
    pub cram: Option<PathBuf>,
    /// Path to assembly FASTA input file (required for CRAM)
    #[arg(long, short = 'a')]
    pub fasta: Option<PathBuf>,
    /// Path to FASTQ file to filter (forward or single reads)
    #[arg(long = "fastq", short = 'f', value_name = "FASTQ")]
    pub fastq1: Option<PathBuf>,
    /// Path to paired FASTQ file to filter (reverse reads)
    #[arg(
        long = "fastq2",
        short = 'r',
        value_name = "FASTQ",
        requires = "fastq1"
    )]
    pub fastq2: Option<PathBuf>,
    /// Suffix to use for output filtered files
    #[arg(long, short = 'S', value_name = "SUFFIX", default_value_t = String::from("filtered"))]
    pub suffix: String,
    /// Flag to output a filtered FASTA file
    #[arg(
        long = "fasta-out",
        short = 'A',
        requires = "fasta",
        default_value_t = false
    )]
    pub fasta_out: bool,
    /// Flag to output filtered FASTQ files
    #[arg(
        long = "fastq-out",
        short = 'F',
        requires = "fastq1",
        default_value_t = false
    )]
    pub fastq_out: bool,
    /// Path to output list of read IDs
    #[arg(long = "read-list", short = 'O', value_name = "TXT")]
    pub read_list: Option<PathBuf>,
}

#[derive(Parser, Debug)]
#[command(group(
    ArgGroup::new("alignment")
        .required(false)
        .args(["bam", "cram"]),
))]
pub struct DepthOptions {
    /// Path to input file containing a list of sequence IDs
    #[arg(long = "list", short = 'i', value_name = "TXT")]
    pub list_file: Option<PathBuf>,
    /// Path to BAM file
    #[arg(long, short = 'b')]
    pub bam: Option<PathBuf>,
    /// Path to CRAM file
    #[arg(long, short = 'c', requires = "fasta")]
    pub cram: Option<PathBuf>,
    /// Path to assembly FASTA input file (required for CRAM)
    #[arg(long, short = 'a')]
    pub fasta: Option<PathBuf>,
    /// Bin size for coverage calculations (use 0 for full contig length)
    #[arg(long = "bin-size", short = 's', default_value_t = 1000)]
    pub bin_size: u32,
    /// Window size for coverage calculations size
    #[arg(long = "window-size", short = 'w', num_args(1..), default_values_t = [1.0], value_parser = window_size_range, action = clap::ArgAction::Append)]
    pub window_size: Vec<f64>,
    /// Output bed file name
    #[arg(long = "output", short = 'O', value_name = "BED")]
    pub output: Option<PathBuf>,
}

pub fn parse() -> Arguments {
    Arguments::parse()
}
