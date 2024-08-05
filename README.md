# Minimizer
Minimizer is a program that is able to minimize the size of files so that they still meet the set requirements.

It is the best suited for minimizing files for fast app, which one iteration takes less than second.

Currently it works only on Linux and require nightly rust compiler.

## How to use
- install rust, clone repo and build project
```
cargo install --path .
```
or just compile it with crates.io
```
cargo install minimizer
```
- run minimizer
```
minimizer --input-file input.txt --output-file output.txt --command "echo {}" --attempts 300 --broken-info "BROKEN"
```
to get info about each argument, read source code or run
```
minimizer --help
```

## Test it
```
echo "ABCDEFGH" > input.txt
echo "gABCDEFFGH" >> input.txt
echo "BCDERF" >> input.txt
echo "ABCD" >> input.txt
echo "BDCE" >> input.txt
```
running 
```
minimizer --input-file input.txt --output-file output.txt --command "cat {}" --attempts 300 --broken-info "AB"
```
will probably give you output.txt with content
```
AB
```
algorithms are not deterministic so not always the same result will be achieved

Using bigger number of attempts will increase the chance of getting smaller output file and will enable additional mode which rely on removing line/byte/char one by one. 

## How it works
At start minimizer reads file and checks if this file returns expected output.

If yes, then app continue to run.

At first app checks if file contains valid utf-8 characters, if yes, then two additional modes are enabled, which works on lines and characters.

Each mode(which works on Vec<> of lines, chars and bytes) at start, tries to remove items from start/end of file.

Later in loop random elements from middle/start/end are removed to check if file still returns expected output.