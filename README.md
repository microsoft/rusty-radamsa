



# rusty-radamsa
Radamsa ported to rust lang

Rusty Radamsa is a general purpose fuzzer. It modifies given sample data
in ways, which might expose errors in programs intended to process
the data. For more information, read the fine manual page, or visit
https://gitlab.com/akihe/radamsa.

Rusty Radamsa was written by Amanda Rousseau (malwareunicorn), based on Radamsa by Aki Helin, initially at OUSPG.

## Requirements:
Supported operating systems:
 * GNU/Linux
 * OpenBSD 
 * FreeBSD
 * Mac OS X
 * Windows

Software requirments:
* Rustlang 1.66
* Cargo

## Building Radamsa
```text
git clone <GitLink>
cd rusty-radamsa
cargo build
```

## Usage: 
```text
rustyradamsa.exe [OPTIONS] [FILE]... [COMMAND]
```

## Commands:
```text
  list
          list mutations, patterns and generators
  help
          Print this message or the help of the given subcommand(s)
```

## Arguments:
```text
  [FILE]...
          file or directory as generator input. example: "./input/* test.bin"
```

## Options:
```text
-s, --seed <SEED>
          random seed (u64, default random)

  -n, --count <COUNT>
          how many outputs to generate (u64)

  -H, --hash <HASH>
          hash algorithm for uniqueness checks (default sha256)

  -p, --patterns <PATTERNS>
          which mutation patterns to use (use list command to see all hashes)

  -m, --mutations <MUTATIONS>
          which mutations to use (use list command to see all mutations)

  -g, --generators <GENERATORS>
          which data generators to use (use list command to see all generators)

  -o, --output <OUTPUT>...
          output pattern

  -C, --checksums <CHECKSUMS>
          maximum number of checksums in uniqueness filter (0 disables)

          [default: 10000]

  -d, --delay <DELAY>
          sleep for n milliseconds between outputs

          [default: 0]

  -T, --truncate <TRUNCATE>
          take only first n bytes of each output (mainly intended for UDP). if truncate is zero, no truncation happens

          [default: 0]

  -S, --seek <SEEK>
          start from given testcase

          [default: 0]

  -v, --verbose
          show progress during generation

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```


## MUTATIONS:
  **DEFAULT:** `ft=2,fo=2,fn,num=5,ld,lds,lr2,li,ls,lp,lr,sr,sd,bd,bf,bi,br,bp,bei,bed,ber,uw,ui=2`

| id |complete | desc |
|---|---|---|
|`ab`|&check;|enhance silly issues in ASCII string data handling|
|`bd`|&check;|drop a byte|
|`bed`|&check;|decrement a byte by one|
|`bei`|&check;|increment a byte by one|
|`ber`|&check;|swap a byte with a random one|
|`bf`|&check;| flip one bit|
|`bi`|&check;| insert a random byte|
|`bp`|&check;| permute some bytes|
|`br`|&check;| repeat a byte|
|`fn`|&check;| likely clone data between similar positions|
|`fo`|&check;| fuse previously seen data elsewhere|
|`ft`|&check;| jump to a similar position in block|
|`ld`|&check;| delete a line|
|`lds`|&check;|delete many lines|
|`li`|&check;| copy a line closeby|
|`lis`|&check;|insert a line from elsewhere|
|`lp`|&check;| swap order of lines|
|`lr`|&check;| repeat a line|
|`lr2`|&check;|duplicate a line|
|`lrs`|&check;|replace a line with one from elsewhere|
|`ls`|&check;| swap two lines|
|`nop`|&check;|do nothing (debug/test)|
|`num`|&check;|try to modify a textual number|
|`sd`|&check;| delete a sequence of bytes|
|`sr`|&check;| repeat a sequence of bytes|
|str|&cross;|try to modify a string|
|`td`|&check;| delete a node|
|`tr`|&check;| repeat a path of the parse tree|
|`tr2`|&check;|duplicate a node|
|`ts1`|&check;|swap one node with another one|
|`ts2`|&check;|swap two nodes pairwise|
|`ui`|&check;| insert funny unicode|
|`uw`|&check;| try to make a code point too wide|
|word|&cross;| try to play with what look like n-byte words or values|
|xp|&cross;| try to parse XML and mutate it|
---
## GENERATORS:
  **DEFAULT:** `random,buffer,file=1000,jump=200,stdin=10000`

| id | comeplete | desc |
|---|---|---|
|`stdin` |&check;| Generator to read data from stdin|
|`file`  |&check;| Generator to read data from a file|
|`tcp`|&check;|Generator to read data from a tcp port|
|`udp`|&check;|Generator to read data from a udp port|
|`buffer`|&check;| Generator to read data from buffer|
|`jump`   |&cross;|Generator jump streamer|
|`random` |&check;|Generator to make random bytes|
|`pcapng` |&cross;|Generator to generate pcapng data|
---
## PATTERNS:
  **DEFAULT:** `od,nd=2,bu`

| id | complete| desc |
|---|---|---|
|`od`|&check;| Mutate once
|`nd`|&check;| Mutate possibly many times
|`bu`|&check;| Make several mutations closeby once
---
## HASHES:
  **DEFAULT:** `sha256`

| id | complete |desc |
|---|---|---|
|`sha`|&check;|Default Hash Sha-256
|`sha256` |&check;|Hash Sha-256
|`sha512` |&check;|Hash Sha-512
|`crc`|&check;|Default CRC-64/CKSUM
|`crc32`  |&check;|CRC-32/CKSUM
|`crc64`  |&check;|CRC-64/REDIS
|`crc82`  |&check;|CRC-82/DARC
---
## OUTPUTS:
  **DEFAULT:** `-`

| id | complete | desc |
|---|---|---|
|`file`| &check; | Write output data to a binary file
|`tcpserver`|&cross;|  Write output data to a tcp port as server
|`tcpclient`|&check;|  Write output data to a tcp port as client
|`udpserver` |&cross;| Write output data to a udp port as server
|`udpclient` |&check;| Write output data to a udp port as client
|`buffer`|&check;| Write output data to a buffer address or vector
|`hash`|&cross;|   Write output variations or a hashing directory using %n and %s as in the template path (i.e. /tmp/fuzz-%n.%s)
|`template` |&cross;|  Output template. %f is fuzzed data. e.g. "<html>%f</html>"
---

## Lib Examples
```
use std::boxed::Box;
extern crate rusty_radamsa;
let data = Box::<[u8]>::from("Hello World 12345689\n".as_bytes());
let mut out_buffer = Box::<[u8]>::from(vec![0u8; 2048]);
let max_len = out_buffer.len(); //aka truncate
let seed: u64 = 42;
let _len = rusty_radamsa::radamsa(&data, data.len(), &mut out_buffer, max_len, seed);
```
> Check out the examples folder for more implementations
## Command Line Examples
List all generators, mutators, patterns, hashes, and outputs options.
```text
rustyradamsa.exe list -a
```
Mutate mutiple files using the num mutator for 100 unique mutations to stdout.
```text
rustyradamsa.exe -g file -m num -n 100 ./tests/hello*
```
Mutate stdin to an out put file.
```text
echo "hello 12345" | rustyradamsa.exe -o file output.bin
```
Generate random data and pipe to a bin file.
```text
rustyradamsa.exe -g random > some.bin
```
Get data from TCP Stream using the num mutator.
```text
rustyradamsa.exe -m num -g tcp "127.0.0.1:6666"
```
Send data to TCP Stream using the random generator.
```text
rustyradamsa.exe -g random -o tcpclient "127.0.0.1:6666"
```
Send data to UDP server. 
```text
rustyradamsa.exe -g random -T 30 -o udpclient 127.0.0.1:8888,127.0.0.1:8000 -v
```
Generate from UDP input.
```text
rustyradamsa.exe -m num -g udp 0.0.0.0:8888 -v
```

## TODOs:
* Seek to test case
* Templated filenames for output
* Template output (--output-template)
* Delay between mutations (--delay)
* Pcapng generator (pcapng)
* Jump generator (jump)
* Saving metadata (--meta)
* Mutator: Xml (xp)
* Mutator: Byte inversion 
* Mutator: Even powers of two
* Mutator: Add/subtract a random value from 0..16
* Mutator: Overwrite contents with zero bytes
* Fix generic mutators to be stateful
* Make the mutation hash global for external use

## Contributing

This project welcomes contributions and suggestions.  Most contributions require you to agree to a
Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us
the rights to use your contribution. For details, visit https://cla.opensource.microsoft.com.

When you submit a pull request, a CLA bot will automatically determine whether you need to provide
a CLA and decorate the PR appropriately (e.g., status check, comment). Simply follow the instructions
provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/).
For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or
contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.

## Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft 
trademarks or logos is subject to and must follow 
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
