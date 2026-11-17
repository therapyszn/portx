# portx

CLI tool to find what process is holding a local TCP port, inspect its working directory and git branch, and terminate it cleanly.

## Features

- List listening developer ports with process name, PID, memory, CWD, and active git branch.
- Inspect single or multiple ports with detailed process hierarchy and launch command.
- Detect Docker container / proxy wrappers.
- Free locked ports (`portx free <ports>`) with safety confirmation or `-f` (`--force`).
- JSON output support (`--json`) for scripting.

## Installation

```bash
cargo build --release
# binary is located at target/release/portx
```

Or install locally:

```bash
cargo install --path .
```

## Usage

### List listening ports

```bash
portx
```

Include system and kernel ports:

```bash
portx -a
```

### Inspect specific ports

```bash
# Detailed single port inspection
portx 3000

# Multiple ports table
portx 3000 8080 5432
```

### Free a port

```bash
# Interactive prompt
portx free 3000

# Force kill without prompt
portx free 3000 -f

# Kill multiple ports at once
portx free 3000 8080 5432 -f
```

### JSON output

```bash
portx 3000 --json
```
