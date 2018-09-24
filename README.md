# HGI image compression #

HGI (Hierarchical Grid Interpolation) is a next generation image compression algorithm, the basic idea of the hierarchical grid interpolation method is to hierarchically decimating the two-dimensional grid of image counts, restoring missing counts via interpolation and statistical coding for interpolation residues.


|Source image | HGI compressed (low)|
|:------------: | :------------: |
![Lena Source Image](docs/static_files/lena_source.png "Lena Source Image") | ![Lena HGI Image](docs/static_files/lena_hgi.png)
156kb|25kb


## Features ##
    * Fixed maximum error of compressed image
    * Several types of image interpolation
    * Several types of statical coding

## Build  ##
```
$ git clone https://github.com/pl0q1n/RustyHGI.git
$ cd RustyHGI 
$ cargo build
```

## Usage ##

```
hgi <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    decode    Decode mode of HGI
    encode    Encode mode of HGI
    help      Prints this message or the help of the given subcommand(s)
    test      Test mode for testing both encode and decode
```

### Encode ###

```
hgi encode [OPTIONS] --input <input> --output <output>

 OPTIONS:
    -i, --input <input>                        Filepath to the source image
    -l, --level <level>                        [default: 4]
    -o, --output <output>                      Output name of compressed image
    -q, --quantizator <quantization_level>     [default: medium]  [possible values: Loseless, Low, Medium, High]
```

### Decode ###

```
hgi decode --input <input> --output <output>

OPTIONS:
    -i, --input <input>                       Filepath to the HGI compressed image
    -o, --output <output>                     Output name of decoded image
```

### Test ###

```
hgi test [OPTIONS] <input>

OPTIONS:
    -l, --level <level>                        Number of levels for hierarchical grid [default: 4]
    -q, --quantizator <quantization_level>     Compression level [default: medium]  [possible values: Loseless, Low, Medium, High]
    -s, --suffix <suffix>                      Suffix for filename [default: ]
```

### References ###

Gashnikov, M.V., Glumov, N.I., Sergeev, V.V. A hierarchical compression method for space images. (Automation and Remote Control, V. 71, No.3, pp. 501-513, 2010) 
