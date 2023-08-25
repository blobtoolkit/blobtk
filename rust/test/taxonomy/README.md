Reduced phylogenies made using

NCBI

```
./target/release/blobtk taxonomy \
    --taxdump ~/projects/genomehubs/boat-resources/taxdump/ncbi \
    -r 9611 \
    -b 1 \
    --taxdump-out test/taxonomy/canidae/ncbi
```

```
./target/release/blobtk taxonomy \
    -g ~/Downloads/backbone-simple.txt \
    -r 9701 \
    -b 1 \
    --taxdump-out test/taxonomy/canidae/gbif
```

Map gbif to ncbi using

```
./target/release/blobtk taxonomy -c test/taxonomy/config.yaml
```
