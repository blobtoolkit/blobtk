# BlobToolKit Core (v0.1.2)

## About

BlobToolKit Core contains a set of core functions used by BlobToolKit tools. Implemented in Rust, these functions are intended to be accessible from the command line, as python modules and web assembly code for use in javascript.

## Installing

### Command line tool

The command line tool is available as a linux/macos binary from the latest release page.

linux:

```
curl -Ls "https://github.com/blobtoolkit/core/releases/download/0.1.2/blobtoolkit-core-linux" > blobtoolkit-core &&
chmod 755 blobtoolkit-core
```

macos:

```
curl -Ls "https://github.com/blobtoolkit/core/releases/download/0.1.2/blobtoolkit-core-macos" > blobtoolkit-core &&
chmod 755 blobtoolkit-core
```

### Python module

The python can be installed using pip:

```
pip install blobtoolkit-core
```

## Usage

### Command line tool

```
./blobtoolkit-core filter --help
Filter files based on list of sequence names

Usage: blobtoolkit-core filter [OPTIONS] <--bam <BAM>|--cram <CRAM>>

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

### Python module

```
from blobtoolkit_core import filter

# filter fastq files based on a list of sequence names
read_count = filter.fastx(
    {
        "list_file": "test/test.list",
        "bam": "test/test.bam",
        "fastq1": "test/reads_1.fq.gz",
        "fastq2": "test/reads_2.fq.gz",
        "fastq_out": True,
    }
)

print(read_count)
```
