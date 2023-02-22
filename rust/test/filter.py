#!/usr/bin/env python3

from blobtk import filter

# filter fastq files based on a list of sequence names
read_count = filter.fastx(
    list_file="test/test.list",
    bam="test/test.bam",
    fastq1="test/reads_1.fq.gz",
    fastq2="test/reads_2.fq.gz",
    fastq_out=True,
)

print(read_count)
