use std::collections::HashSet;
use std::path::PathBuf;

use clap::{ArgGroup, Parser, Subcommand, ValueEnum};
use pyo3::pyclass;

// fn float_range(s: &str, min: f64, max: f64) -> Result<f64, String> {
//     debug_assert!(min <= max, "minimum of {} exceeds maximum of {}", min, max);
//     let val = match s.parse::<f64>() {
//         Ok(v) => v,
//         Err(e) => panic!("{:?}", e),
//     };
//     check_float_range(val, min, max)
// }

// fn check_float_range(val: f64, min: f64, max: f64) -> Result<f64, String> {
//     if val > max {
//         Err(format!("exceeds maximum of {}", max))
//     } else if val < min {
//         Err(format!("less than minimum of {}", min))
//     } else {
//         Ok(val)
//     }
// }

// fn window_size_range(s: &str) -> Result<f64, String> {
//     float_range(s, 0.0001, 1000000000.0)
// }

fn bin_size_parser(s: &str) -> Result<usize, String> {
    let mut val = match s.parse::<usize>() {
        Ok(v) => v,
        Err(e) => panic!("{:?}", e),
    };
    if val == 0 {
        val = usize::MAX
    }
    Ok(val)
}

/// Top level arguments to `blobtk`
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// `blobtk` subcommands
#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// Calculate sequencing coverage depth.
    /// Called as `blobtk depth`
    Depth(DepthOptions),
    /// Filter files based on list of sequence names.
    /// Called as `blobtk filter`
    Filter(FilterOptions),
    /// Process a BlobDir and produce static plots.
    /// Called as `blobtk plot`
    Plot(PlotOptions),
    /// Process a taxonomy and lookup lineages.
    /// Called as `blobtk taxonomy`
    Taxonomy(TaxonomyOptions),
}

/// Options to pass to `blobtk depth`
#[derive(Parser, Debug)]
#[command(group(
    ArgGroup::new("alignment")
        .required(false)
        .args(["bam", "cram"]),
))]
#[pyclass]
pub struct DepthOptions {
    /// List of sequence IDs
    #[clap(skip)]
    pub list: Option<HashSet<Vec<u8>>>,
    /// Path to input file containing a list of sequence IDs
    #[arg(long = "list", short = 'i', value_name = "TXT")]
    pub list_file: Option<PathBuf>,
    /// Path to BAM file
    #[arg(long, short = 'b')]
    pub bam: Option<PathBuf>,
    /// Path to CRAM file
    #[arg(long, short = 'c')]
    pub cram: Option<PathBuf>,
    /// Path to assembly FASTA input file (required for CRAM)
    #[arg(long, short = 'a')]
    pub fasta: Option<PathBuf>,
    /// Bin size for coverage calculations (use 0 for full contig length)
    #[arg(long = "bin-size", short = 's', default_value_t = 0, value_parser = bin_size_parser)]
    pub bin_size: usize,
    // /// Window size for coverage calculations size
    // #[arg(long = "window-size", short = 'w', num_args(1..), default_values_t = [1.0], value_parser = window_size_range, action = clap::ArgAction::Append)]
    // pub window_size: Vec<f64>,
    /// Output bed file name
    #[arg(long = "bed", short = 'O', value_name = "BED")]
    pub bed: Option<PathBuf>,
}

/// Options to pass to `blobtk filter`
#[derive(Parser, Debug)]
#[command(group(
    ArgGroup::new("alignment")
        .required(false)
        .args(["bam", "cram"]),
))]
#[pyclass]
pub struct FilterOptions {
    // TODO: add option to invert list (use BAM header)
    /// List of sequence IDs
    #[clap(skip)]
    pub list: Option<HashSet<Vec<u8>>>,
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

#[derive(ValueEnum, Clone, Debug)]
pub enum View {
    Blob,
    Cumulative,
    Snail,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Palette {
    Default,
    Inverse,
    Viridis,
}

/// Options to pass to `blobtk plot`
#[derive(Parser, Debug)]
#[pyclass]
pub struct PlotOptions {
    /// Path to BlobDir directory
    #[arg(long, short = 'd')]
    pub blobdir: PathBuf,
    /// View to plot
    #[arg(long, short = 'v')]
    #[clap(value_enum)]
    pub view: Option<View>,
    /// Output filename
    #[arg(long, short = 'o', default_value_t = String::from("output.svg"))]
    pub output: String,
    #[arg(long, short = 'f')]
    pub filter: Vec<String>,
    /// Segment count for snail plot
    #[arg(long, short = 's', default_value_t = 1000)]
    pub segments: usize,
    /// Max span for snail plot
    #[arg(long = "max-span")]
    pub max_span: Option<usize>,
    /// max scaffold length for snail plot
    #[arg(long = "max-scaffold")]
    pub max_scaffold: Option<usize>,
    /// X-axis field for blob plot
    #[arg(long = "x-field", short = 'x')]
    pub x_field: Option<String>,
    /// Y-axis field for blob plot
    #[arg(long = "y-field", short = 'y')]
    pub y_field: Option<String>,
    /// Z-axis field for blob plot
    #[arg(long = "z-field", short = 'z')]
    pub z_field: Option<String>,
    /// Category field for blob plot
    #[arg(long = "category", short = 'c')]
    /// Category field for blob plot
    #[arg(long = "category", short = 'c')]
    pub cat_field: Option<String>,
    /// Resolution for blob plot
    #[arg(long, default_value_t = 30)]
    pub resolution: usize,
    /// Maximum number of categories for blob/cumulative plot
    #[arg(long = "cat-count", default_value_t = 10)]
    pub cat_count: usize,
    /// Category order for blob/cumulative plot (<cat1>,<cat2>,...)
    #[arg(long = "cat-order")]
    pub cat_order: Option<String>,
    /// Colour palette for categories
    #[arg(long, value_enum)]
    pub palette: Option<Palette>,
    /// Individual colours to modify palette (<index>=<hexcode>)
    #[arg(long)]
    pub color: Option<Vec<String>>,
}

/// Options to pass to `blobtk taxonomy`
#[derive(Parser, Debug)]
#[pyclass]
pub struct TaxonomyOptions {
    /// Path to NCBI taxdump directory
    #[arg(long, short = 't')]
    pub taxdump: Option<PathBuf>,
    /// Root taxon to build taxonomy for
    #[arg(long = "root-id", short = 'r')]
    pub root_id: Option<String>,
}

/// Command line argument parser
pub fn parse() -> Arguments {
    Arguments::parse()
}
