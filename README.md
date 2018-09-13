Idash challenge task 2 submission by T.Tortech & S.Carpov (CEA)
thibaud.tortech and sergiu.carpov at cea.fr

Our solution splits the analysis algorithm in two steps:
    1. Compression\&encryption of input vcf files (this step is supposed to be done by the owner of the data at a protocol level).
    2. Analysis algorithm itself is performed on a public server with SGX support.

Shell file 'run.sh' contains sample commands. Variables in the shell file can be used to configure input/output data paths.

1. Input vcf files are compressed and encrypted using './ce' binary.
AES-128 (GCM mode) encryption is used. Secret key is hard-coded. openssl library is used. Please install libssl.
Input case, control paths and output directory can be configured using command-line arguments.

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


2. Analysis of encrypted data is performed using './app' binary which loads enclave in 'enclave.signed.so'.
Top-k most SNP alleles are written by default in file 'idashChisq.vcf' (can be changed using -f argument).
The number top alleles to find is configured using '-k' argument. For generating allele frequency vcf file use '-a'.
To ease results interpretation (and avoid implementing a decryption binary :)) output files are written in clear.
Input case and control paths are set using '-c' and '-C' flags.

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

If you have any question don't hesitate.
