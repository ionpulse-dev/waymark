# Development

waymark is built and tested on macOS with Apple silicon and on Linux (x86_64 and arm64). The work follows the spec-driven process in [specs/WORKFLOW.md](../specs/WORKFLOW.md).

## On macOS or Linux

Clone the repository and run:

```bash
make setup
make check
```

`make setup` checks what is missing. On macOS it can install `rustup` with Homebrew. On Linux it never runs a package manager: it tells you the command (you need `git` 2.28 or newer, `make`, a C compiler, and `rustup`; on Ubuntu or Debian: `sudo apt install git build-essential`, then install `rustup` from <https://rustup.rs>). `make doctor` shows what is ready.

`make help` lists every command: build, test, lint, run, and release preparation.

## In a Linux container (on a Mac)

The maintainer works on a Mac. Everything also runs in a Linux container, so you can test on Linux without a Linux computer. The container is a development tool only. It is not the release image of a server.

You need a container runtime with a Docker-compatible command. The tested one is [OrbStack](https://orbstack.dev). Check the license terms for your own use. Other runtimes are best effort.

```bash
make linux-test      # all tests in the container
make linux-check     # lint, tests, and spec check in the container
make linux-build     # debug build in the container
make linux-shell     # a shell in the container
```

Run as x86_64 on an Apple silicon Mac with `ARCH=amd64 make linux-test` (slower, because it is emulated). CI runs x86_64.

How it works:

- `.devcontainer/Dockerfile` holds the toolchain (Ubuntu 22.04, `git`, and the Rust version of `rust-toolchain.toml`). It holds no project file.
- `scripts/linux.sh` builds the image, mounts the source at `/work`, and runs your command as your user ID, so no file in the project folder is owned by `root`.
- `target/` and the Cargo caches live in named volumes, not in your mounted folder, so builds are fast.
- Tests make their folders in the temp folder of the container, which is case-sensitive.
- `.devcontainer/devcontainer.json` lets an editor attach to the same container.

If the runtime is not running, the commands say: "Start OrbStack (or your container runtime) and try again."

## For AI agents on a Mac

The full suite runs after every task. On a Mac, run `make test-all` on the host, and `make linux-test` for the Linux result. If `make doctor` says there is no running container runtime, say so and use the host.
