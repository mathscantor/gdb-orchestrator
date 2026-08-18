# gdb-orchestrator

A tool for debugging multiple processes simultaneously via pre-defined GDB commands. 

## 1. Quick Start

**When to use:** Automate debugging of multiple processes (e.g., fork-heavy binaries) instead of manually managing multiple GDB instances.

Statically compiled binaries are provided for multiple architectures in latest [release](https://github.com/mathscantor/gdb-orchestrator/releases/latest):
- **gdborch**: The main gdb-orchestrator tool
- **gdb** & **gdbserver** (v17.1): From [gdb-static](https://github.com/guyush1/gdb-static) with built-in Python support

| Architecture | Supported? |
|--------------|------------|
| x86_64       | ✓          |
| i686         | ✗          |
| aarch64      | ✓          |
| arm          | ✓          |
| mips         | ✗          |
| mipsel       | ✗          |
| powerpc      | ✗          |

## 2. Building

### 2.1.1 Prerequisites
- [Docker](https://docs.docker.com/engine/install) or [Podman](https://podman.io/docs/installation)
- [Cross](https://github.com/cross-rs/cross) for cross-compilation

### 2.1.2 Build
```bash
# Install cross
cargo install cross

# x86_64
cross build --release --target x86_64-unknown-linux-musl

# aarch64
cross build --release --target aarch64-unknown-linux-musl

# arm
cross build --release --target arm-unknown-linux-musleabi
```

## 3. How It Works

gdb-orchestrator spawns multiple GDB / GDB Server processes in the background. Sessions are organized by timestamp (`DD-MM-YYYYTHH:mm:ss`) and stored in `.gdborch/` with outputs and a SQLite3 database (`gdborch.db`) for tracking.

## 4. Configuration

**Default paths**:
- GDB: `/usr/bin/gdb`
- GDB Server: `/usr/bin/gdbserver`

**Custom paths using environment variables binaries are installed elsewhere:**
```bash
GDB_PATH=/opt/gdb gdborch ...
GDBSERVER_PATH=/opt/gdbserver gdborch ...
```

## 5. Commands

### 5.1 Local Debugging
```bash
# Attach to all 'test' processes
gdborch client local start -n test -s gdb_scripts/test.gdb

# Attach to specific PIDs
gdborch client local start -p 44917,44906 -s gdb_scripts/test.gdb

# List sessions
gdborch client local show

# Stop a session
gdborch client local stop -s 29-01-2026T00-18-28

# Stop all sessions
gdborch client local stop -a
```

### 5.2 Remote Debugging

**Server (Machine with target processes):**
```bash
# Spawn 3 gdbserver instances listening on ports 5555-5557
gdborch server start -n test
```

**Client (Debugging machine):**
```bash
# Connect to remote processes
gdborch client remote start -c 192.168.1.100:5555,192.168.1.100:5556,192.168.1.100:5557 -s gdb_scripts/test.gdb
```

## 6. FAQ

**Why rewrite from Bash to Rust?**

The original Bash version assumed a bash shell, but restricted environments (like embedded systems) often only provide sh. Rather than downgrade to pure sh, Rust was chosen for better portability across multiple compiler toolchains and architectures.

## 7. Bug Report
To report a bug, kindly include the following when opening an issue:
1. Provide a brief description of observed bug.
2. Make sure to replicate the bug with `--verbose` option set to show DEBUG logs
3. Make sure to show to full command line
