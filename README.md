# Minimizer
Minimizer is a program that is able to minimize the size of files so that they still meet the set requirements.

It is the best suited for minimizing files for fast app, which one iteration takes less than second.

Currently it works only on Linux.

## How to use
- install rust on linux, clone repo and build project
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
minimizer --input-file input.txt --output-file output.txt --command "cat {}" --attempts 300 --broken-info "AB" -e -v
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

## Different strategies
Basing on different files, different strategies can be used to minimize file.

In repo only one general strategy is implemented, which should be good for most of the files.

But if you have some specific file, you can implement your own strategy 

## Typical commands
### Ruff
```
minimizer --input-file /home/rafal/Desktop/RunEveryCommand/C/PY_FILE_TEST_25518.py --output-file a.py --command "red_knot" --attempts 1000 --broken-info "RUST_BACKTRACE" -z "not yet implemented" -z "failed to parse" -z "SyntaxError" -z "Sorry:" -z "IndentationError" -k "python3 -m compileall {}" -r -v
```
or shorter
```
minimizer -i /home/rafal/Desktop/RunEveryCommand/C/PY_FILE_TEST_25518.py -o a.py -c "red_knot" -a 1000 -b "RUST_BACKTRACE" -z "not yet implemented" -z "failed to parse" -z "SyntaxError" -z "Sorry:" -z "IndentationError" -k "python3 -m compileall {}" -r -v
```

### Red Knot
```
minimizer --input-file /home/rafal/Desktop/RunEveryCommand/C/PY_FILE_TEST_25518.py --output-file a.py --command "red_knot" --attempts 1000 --broken-info "RUST_BACKTRACE" -z "not yet implemented" -z "failed to parse" -z "SyntaxError" -z "Sorry:" -z "IndentationError" -k "python3 -m compileall {}" -r -v
```

### Lofty
```
minimizer --input-file input.mp3 --output-file output.mp3 --command "lofty {}" --attempts 100000 -r --broken-info "RUST_BACKTRACE" -v --max-time 200 --strategy pedantic
```
or sho
```
minimizer -i input.mp3 -o output.mp3 -c "lofty {}" -a 100000 -r -b "RUST_BACKTRACE" -v -t 200 -s pedantic
```

## Why
I just needed this - I doubt that it will be useful for anyone else, but feel free to use this.

## License
MIT License