#!/bin/bash

echo "Runnning integration tests"

CMD="./target/release/blobtk depth -b test/test.bam -O test/test.bed"
printf "\nrunning command\n$CMD\n\n"
$CMD || exit 1

CMD="./target/release/blobtk depth -b test/test.bam -s 1000 -O test/test.1000.bed"
printf "\n\nrunning command\n$CMD\n\n"
$CMD || exit 1

CMD="./target/release/blobtk filter -i test/test.list -b test/test.bam -f test/reads_1.fq.gz -r test/reads_2.fq.gz -F"
printf "\n\nrunning command\n$CMD\n\n"
$CMD || exit 1

CMD="./test/depth.py"
printf "\n\nrunning command\n$CMD\n\n"
$CMD || exit 1

CMD="./test/filter.py"
printf "\n\nrunning command\n$CMD\n\n"
$CMD || exit 1

printf "\nFinished running integration tests\n\n"