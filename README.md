# BlobTk (v0.4.5)

## About

BlobTk contains a set of core functions used by BlobToolKit tools. Implemented in Rust, these functions are intended to be accessible from the command line, as python modules and will include web assembly code for use in javascript.

## Installing

### Command line tool

The command line tool is available as a linux/macos binary from the latest release page.

linux:

```
curl -Ls "https://github.com/blobtoolkit/core/releases/download/0.4.5/blobtk-linux" > blobtk &&
chmod 755 blobtk
```

macos:

```
curl -Ls "https://github.com/blobtoolkit/core/releases/download/0.4.5/blobtk-macos" > blobtk &&
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
blobtk filter --help

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

#### `blobtk plot`

```
blobtk plot -h

Process a BlobDir and produce static plots. Called as `blobtk plot`

Usage: blobtk plot [OPTIONS] --blobdir <BLOBDIR>

Options:
  -d, --blobdir <BLOBDIR>
          Path to BlobDir directory
  -v, --view <VIEW>
          View to plot [possible values: blob, cumulative, snail]
  -o, --output <OUTPUT>
          Output filename [default: output.svg]
  -f, --filter <FILTER>

  -s, --segments <SEGMENTS>
          Segment count for snail plot [default: 1000]
      --max-span <MAX_SPAN>
          Max span for snail plot
      --max-scaffold <MAX_SCAFFOLD>
          max scaffold length for snail plot
  -x, --x-field <X_FIELD>
          X-axis field for blob plot
  -y, --y-field <Y_FIELD>
          Y-axis field for blob plot
  -z, --z-field <Z_FIELD>
          Z-axis field for blob plot
  -c, --category <CAT_FIELD>
          Category field for blob plot
      --resolution <RESOLUTION>
          Resolution for blob plot [default: 30]
      --hist-height <HIST_HEIGHT>
          Maximum histogram height for blob plot
      --reducer-function <REDUCER_FUNCTION>
          Reducer function for blob plot [default: sum] [possible values: sum, max, min, count, mean]
      --scale-function <SCALE_FUNCTION>
          Scale function for blob plot [default: sqrt] [possible values: linear, sqrt, log]
      --scale-factor <SCALE_FACTOR>
          Scale factor for blob plot (0.2 - 5.0) [default: 1]
      --x-limit <X_LIMIT>
          X-axis limits for blob/cumulative plot (<min>,<max>)
      --y-limit <Y_LIMIT>
          Y-axis limits for blob/cumulative plot (<min>,<max>)
      --cat-count <CAT_COUNT>
          Maximum number of categories for blob/cumulative plot [default: 10]
      --legend <SHOW_LEGEND>
          Maximum number of categories for blob/cumulative plot [default: default] [possible values: default, full, compact, none]
      --cat-order <CAT_ORDER>
          Category order for blob/cumulative plot (<cat1>,<cat2>,...)
      --origin <ORIGIN>
          Origin for category lines in cumulative plot [possible values: o, x, y]
      --palette <PALETTE>
          Colour palette for categories [possible values: default, inverse, viridis]
      --color <COLOR>
          Individual colours to modify palette (<index>=<hexcode>)
  -h, --help
          Print help
```

Blob plot (as png):

```
blobtk plot -v blob -d /path/to/BlobDir -o blob_plot_filename.png
```

Cumulative plot (as png):

```
blobtk plot -v cumulative -d /path/to/BlobDir -o cumulative_plot_filename.png
```

Snail plot (as svg):

```
blobtk plot -v snail -d /path/to/BlobDir -o snail_plot_filename.svg
```

### `blobtk taxonomy`

Filter NCBI taxonomy for Canidae

```
blobtk taxonomy -t /path/to/ncbi/taxdump -r 9611 -b 1 --taxdump-out /tmp/ncbi-canidae
```

Filter GBIF taxonomy for Canidae

```
./target/release/blobtk taxonomy -g ~/Downloads/gbif/backbone-simple.txt -r 9701 -b 1 --taxdump-out /tmp/gbif-canidae
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
