# SGNTX - SGX based whole genome variants search

SGNTX (pronounced "sgenetics") is a prototype for secure whole genome variants search based on Intel SGX.

It was developed by S. Carpov and T. Tortech for the 2nd track of [iDASH Privacy & Security Workshop 2017](http://www.humangenomeprivacy.org/2017/) competition.
The goal of the competition was to develop scalable solutions using secure hardware (i.e., SGX) to enable secure whole genome variants search among multiple individuals.
In particular, the search application was to find the top most significant SNPs (Single-Nucleotide Polymorphisms) in a database of genome records labeled with *control* or *case*.

It had been selected as the best submission amongst other competition entries and was awarded the first prize :clap: :clap: :clap:

## Solution details

The analysis algorithm is split into 2 steps:

1. **compress & encrypt** input vcf (variant call format) files,

1. **analysis** algorithm performed on a public server with SGX support.

#### Compress & encrypt application

Input vcf files are compressed and encrypted using the `./ce` binary.
Each SNP from input file is compressed into a 10-byte binary format.
Blocks of binary encoded SNPs are then encrypted using `openssl` library.
AES-128 (GCM mode) encryption is used. Secret key is hard-coded.

Input case, control paths and output directory can be configured using command-line arguments:

```
$ ./ce -h
-=< Compression & Encryption >=-
ce 0.1

USAGE:
    ce [OPTIONS] --case <DIR> --control <DIR>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --case <DIR>        Case .vcf directory
    -C, --control <DIR>     Control .vcf directory
    -o, --out_path <STR>    Output directory [default: ]
```

#### Analysis application

Analysis of encrypted data is performed using `./app` which employs the enclave module from file `enclave.signed.so`.

Top-k most significant SNPs are written by default to file `idashChisq.vcf` (can be changed using `-f` argument).
The number top SNPs to find is configured using `-k` argument.
For generating allele frequency file use `-a` flag.
Input case and control paths containing `.vcf` files are set using `-c` and respectively `-C` arguments.

```
$ ./app -h
-=< Analysing >=-
app 0.1

USAGE:
    app [FLAGS] [OPTIONS] --case <DIR> --control <DIR>

FLAGS:
    -h, --help                  Prints help information
    -a, --output_allele_freq    Output allele frequecies
    -V, --version               Prints version information

OPTIONS:
    -c, --case <DIR>         Case .vcf directory
    -C, --control <DIR>      Control .vcf directory
    -f, --output <STR>       Prefix of output files [default: ]
    -k, --snp_count <INT>    Count of top SNP alleles to compute [default: 10]
```

To ease results interpretation (and avoid implementing a decryption binary :smile:) output files are written in clear.


## Implementation details

Rust programming language was used to implement both applications (`./ce` and
`./app`). Enclave part uses the [Rust-SGX SKD](https://github.com/baidu/rust-
sgx-sdk) to interface Intel SGX, in particular the docker image it provides.


## Compilation

We suppose Intel SGX drivers were already installed. See https://01.org/intel-softwareguard-extensions for more details.

Clone `sgntx` repository:
```
git clone https://github.com/CEA-LIST/sgntx.git
```

Update `rust-sgx-sdk` submodule to v1.0.0 tag:
```
git submodule update
cd rust-sgx-sdx
git checkout v1.0.0
cd ..
```

Run [`baiduxlab/sgx-rust`](https://hub.docker.com/r/baiduxlab/sgx-rust/) version 1.0.0 docker image:

```
docker run \
-v $PWD/rust-sgx-sdk:/root/sgx -v $PWD:/root/idash \
-ti --device /dev/isgx --name sgx_idash --network host \
baiduxlab/sgx-rust:1.0.0
```

Inside container compile:
```
cd idash
make
```

## Further details

More information about solution and execution
performance can be found in paper "Carpov, S., & Tortech, T. (2018). Secure
top most significant genome variants search: iDASH 2017 competition. *BMC
medical genomics*, 11(4), 82." available here
https://doi.org/10.1186/s12920-018-0399-x.

Typical runtime on a sample dataset of 27GB is under 1 minute. The majority of
time is used (50 seconds) to compress & encrypt data and only 6 seconds for
the analysis part. Compressed dataset is 5x smaller (5.5GB) than the input
one. A desktop PC with an Intel(R) Xeon(R) CPU E3-1240 (3.50GHz) processor
with 16 GB of RAM was used for this benchmark and a RAM disk was used for
intermediary data.
