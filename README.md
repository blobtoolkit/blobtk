# BlobToolKit Core (v0.1.1)

## About

BlobToolKit Core contains a set of core functions used by BlobToolKit tools. Implemented in Rust, these functions are intended to be accessible from the command line, as python modules and web assembly code for use in javascript.

## Usage

### Python

```
pip install blobtoolkit-core

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
