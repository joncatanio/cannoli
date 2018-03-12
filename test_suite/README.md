# Python3 Test Suite

This is a custom test suite created to easily create Python files and test
them against Cannoli. Instead of hacking together a solution using Rust's
integration tests to diff Python output, I just wrote a quick script.

## Requirements
- Build Cannoli by running `cargo build` in the `cannoli` root directory. This
  will enable the symlink included in this directory.
- Clone [Cannolib](https://github.com/joncatanio/cannolib) and place it in the
  same directory with Cannoli. This allows the `Cargo.toml` file in `sandbox`
  to find the Cannolib library.

## How to Run Tests
`run_tests` is a very simple `bash` script which runs all tests located in
the `suite/` directory when invoked as so `./run_tests`. When no arguments are
provided it will run through each `suite/` subdirectory, print out `info.txt`,
run `test.py` in Python and Cannoli, then compare the output.

Since `cannolib` is used it is much easier to compile the Rust files with
`cargo`. Therefore the `sandbox` directory allows `run_tests` to drop the `*.rs`
files, output from Cannoli, into `sandbox/src` subsequently running
`cargo build` from `sandbox` to build the Cannoli executable.

### Running Individual Tests
Simply invoke the `run_tests` script with a test directory to be evaluated.

Ex: `./run_tests test01`

## Creating Custom Tests
- Create new test directory under `suite` [ex: `mkdir suite/test01`]
- Add an `info.txt` file which explains what the test does
  [ex: `echo "simple test" > suite/test01/info.txt`]
- Create `test.py` as the main Python file, other modules may be added and used
