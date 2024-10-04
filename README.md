
---

# watcher_c

`watcher_c` is a simple CLI tool written in Rust that watches a C source file for changes, automatically recompiles it using `gcc`, and runs the resulting executable. It supports optional clearing of the terminal screen before each recompilation and displays a message prompting the user to edit the file again after the program execution ends.

## Features

- Watches a specified C file for changes and automatically recompiles it using `gcc`.
- Runs the compiled binary after successful compilation.
- Optionally clears the terminal screen before each recompilation with the `-c` flag.
- Outputs a message prompting you to "edit file to recompile" after the program finishes.
- Customizable delay between recompilations to prevent rapid file changes from triggering too quickly.

## Installation

### Prerequisites

- **Rust**: Make sure you have the Rust toolchain installed. You can install it from [here](https://www.rust-lang.org/tools/install).
- **GCC**: Ensure that `gcc` is installed on your system to compile C code. You can install it via your system's package manager.

### Steps

1. Clone the repository or copy the `watcher_c` code into a local directory.
2. Build the binary:

```bash
cargo build --release
```

3. Optionally, move the compiled binary to a directory in your system’s PATH:

```bash
sudo mv target/release/watcher_c /usr/local/bin/
```

## Usage

### Basic Usage

To watch and recompile a C file whenever it changes:

```bash
watcher_c <file.c>
```

Example:

```bash
watcher_c hello.c
```

This will watch `hello.c`, and when it changes, `gcc` will recompile it, then the tool will run the program.

### Flags

#### `-c`, `--clear`

Clear the terminal screen before each recompilation. Example:

```bash
watcher_c -c hello.c
```

#### `-d`, `--delay`

Set a delay (in seconds) before recompiling after detecting a file change. This can prevent recompiling too rapidly on fast edits. Example:

```bash
watcher_c -d 2 hello.c
```

### Output

- On successful compilation, the binary will be run immediately.
- The program will display:

  ```
  ==================
  Program finished successfully. Edit file to recompile.
  ```

## Example

```bash
watcher_c -c -d 1 hello.c
```

This command will:
- Watch the file `hello.c` for changes.
- Clear the terminal screen before each recompilation (`-c` flag).
- Wait for 1 second before recompiling if the file changes (`-d 1` flag).

## License

This project is licensed under the MIT License.

---

