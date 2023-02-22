# BlobTk (v0.2.1)

## About

BlobTk contains a set of core functions used by BlobToolKit tools. Implemented in Rust, these functions are intended to be accessible from the command line, as python modules and will include web assembly code for use in javascript.

## Installing

### Command line tool

The command line tool is available as a linux/macos binary from the latest release page.

linux:

```
curl -Ls "https://github.com/blobtoolkit/core/releases/download/0.2.1/blobtk-linux" > blobtk &&
chmod 755 blobtk
```

macos:

```
curl -Ls "https://github.com/blobtoolkit/core/releases/download/0.2.1/blobtk-macos" > blobtk &&
chmod 755 blobtk
```

### Python module

The python module can be installed using pip:

```
pip install blobtk
```

## Usage

### Command line tool

#### `blobtk depth`

```
blobtk depth --help
Calculate sequencing coverage depth

Usage: blobtk depth [OPTIONS]

Options:
  -i, --list <TXT>           Path to input file containing a list of sequence IDs
  -b, --bam <BAM>            Path to BAM file
  -c, --cram <CRAM>          Path to CRAM file
  -a, --fasta <FASTA>        Path to assembly FASTA input file (required for CRAM)
  -s, --bin-size <BIN_SIZE>  Bin size for coverage calculations (use 0 for full contig length) [default: 1000]
  -O, --bed <BED>            Output bed file name
  -h, --help                 Print help information
```

```
blobtk depth -b test/test.bam -O test/test.bed

blobtk depth -b test/test.bam -s 1000 -O test/test.1000.bed
```

#### `blobtk filter`

```
./blobtk filter --help
Filter files based on list of sequence names

Usage: blobtk filter [OPTIONS] <--bam <BAM>|--cram <CRAM>>

Options:
  -i, --list <TXT>       Path to input file containing a list of sequence IDs
  -b, --bam <BAM>        Path to BAM file
  -c, --cram <CRAM>      Path to CRAM file
  -a, --fasta <FASTA>    Path to assembly FASTA input file (required for CRAM)
  -f, --fastq <FASTQ>    Path to FASTQ file to filter (forward or single reads)
  -r, --fastq2 <FASTQ>   Path to paired FASTQ file to filter (reverse reads)
  -S, --suffix <SUFFIX>  Suffix to use for output filtered files [default: filtered]
  -A, --fasta-out        Flag to output a filtered FASTA file
  -F, --fastq-out        Flag to output filtered FASTQ files
  -O, --read-list <TXT>  Path to output list of read IDs
  -h, --help             Print help information
```

```
blobtk filter -i test/test.list -b test/test.bam -f test/reads_1.fq.gz -r test/reads_2.fq.gz -F
```

### Python module

#### depth

```
from blobtk import depth

# generate bed file of coverage depths
depth.bam_to_bed(bam="test/test.bam", bed="test/pytest.bed")
depth.bam_to_bed(bam="test/test.bam", bin_size=1000, bed="test/pytest.1000.bed")

binned_covs = depth.bam_to_depth(bam="test/test.bam")
for cov in binned_covs:
    print({cov.seq_name: cov.bins[0]})


```

#### filter

```
from blobtk import filter

# filter fastq files based on a list of sequence names
read_count = filter.fastx(list_file="test/test.list", bam="test/test.bam", fastq1="test/reads_1.fq.gz", fastq2="test/reads_2.fq.gz", fastq_out=True)

print(read_count)
```
