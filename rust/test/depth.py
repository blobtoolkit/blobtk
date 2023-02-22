#!/usr/bin/env python3

from blobtk import depth

# generate bed file of coverage depths
depth.bam_to_bed(bam="test/test.bam", bed="test/pytest.bed")

# generate bed file of coverage depths in 1kb bins
depth.bam_to_bed(bam="test/test.bam", bin_size=1000, bed="test/pytest.1000.bed")

# get list of coverage information from a bam file
binned_covs = depth.bam_to_depth(bam="test/test.bam")
for cov in binned_covs:
    print({cov.seq_name: cov.bins[0]})
