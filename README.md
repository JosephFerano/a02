## Memory Management in Rust for CCOM4017

The code is organized into 4 files; `wsclock.rs`, `second.rs`, `optimal.rs`, and `lib.rs`. Rust automagically generates binaries for the first 3, lib has some shared code.

#### Running

```./optimal <PHYSICAL_MEMORY_PAGES> <FILENAME>```

```./second <PHYSICAL_MEMORY_PAGES> <FILENAME>```

```./wsclock <PHYSICAL_MEMORY_PAGES> <TAU> <FILENAME>```

#### Building

If you wish to compile the code, install rust and cargo
Link(https://www.rust-lang.org/en-US/install.html)

Then just run build
```cargo build```

If you wish to run a specific algorithm, use `run`
```cargo run --bin optimal 10 input.txt```

#### Testing

Each algorithm has over half a dozen tests to ensure that it's behaving correctly. Rust is great because unit testing is baked into the language and local to the module you're working on. The compiler is smart enough to ignore compiling them. In order to facilitate unit testing, the `process_page_requests()` function for each program was separated out so all it's parameters were passed in as parameters. An enum called `AccessResult` also tracks the behavior of the page hits/misses, allowing me to assert that the algorithms are adding/removing pages correctly. The names of the of the tests somewhat describe the intent.

To run the tests, call
```cargo test```

To run tests for a specific program, use
```cargo test --bin wsclock```
No need to use parameters since the tests construct the environment themselves

