>[!NOTE]
>I am working on a [faster v3](https://github.com/brownben/bang3)  

<img src="./logo.svg" height="175px">

# Bang

My attempt at creating my own language. A strongly typed, reference counted, bytecode interpreter written in Rust. Based on the syntax and style of the language I have liked using, such as Python, and TypeScript. Complete with a custom opinionated code formatter, linter, and bidirectional type-checker.

Based on and inspired by the awesome [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom.

View the previous version (a tree walk interpreter in TypeScript) [here](https://github.com/brownben/bang/releases/tag/JS).

### Examples

```bang
// Recursive Fibonacci

let fib_recursive = (n: number) -> number
  if (n <= 2)
    if (n == 0) return 0
    return n - 1
  else return fib_recursive(n - 1) + fib_recursive(n - 2)
```

A quick walkthrough of the language can be found [here](/examples/syntax.bang).

More examples can be found in the [/examples](./examples/) folder.

### Usage

```
Usage: bang.exe [COMMAND]

Commands:
             Open a REPL
  run        Execute a Bang program
  lint       Run linter on a bang file
  format     Format a bang file
  typecheck  Run typechecker on on a file
  print      Print debugging information
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

### Development

```sh
# To format the codebase:
cargo fmt

# To lint the codebase:
cargo clippy

# To run the tests:
cargo test

# To build Bang:
cargo build --release
```

### License

The code in this repository is covered by the [MIT License](./LICENSE).
