# empirical_splice_status

### building

```
cargo build --release
```

### indexing 

```
./target/release/build_index <input_fasta> <output_index_stem>
```

This will create 2 files, `<output_index_stem>.btex` and `<output_index_stem>.idx`.

### mapping / ambiguity determination

```
./target/release/build_index <index_stem> <fastq_file>
```
This will write a 2 column output stream to stdout of the form 

```
read_name splicing_status
```

where `read_name` is the input record header for each entry in `<fastq_file>` and 
`splicing_status` is one of `m`, `n`, `a`, corresponding to which specific set of 
targets the read sequence mapped.
