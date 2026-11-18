# Installation

ACRYL microVMs ships as a command (`acrylmicrovm`), a Node.js SDK and a Rust library. Every way below installs the
same engine: each box is a micro-VM with its own kernel, started from an ordinary container image.

| You want | Fastest way |
| --- | --- |
| Try it once, nothing installed | `npx acrylmicrovm run --rm alpine:latest echo hello` |
| The command on your machine | `cargo binstall acrylmicrovm-cli`, `cargo install acrylmicrovm-cli` or the [install script](#install-script) |
| Boxes from JavaScript or TypeScript | `npm i acrylmicrovm` (or pnpm, yarn, bun, deno) |
| Boxes from Rust | `cargo add acrylmicrovm` |

Every command on this page was run against the published packages on macOS (Apple Silicon) and Linux (x86_64).

## Prerequisites

| Platform | Architecture | Status | Needs |
| --- | --- | --- | --- |
| macOS | Apple Silicon (arm64) | Supported | Hypervisor.framework (built in) |
| Linux | x86_64 | Supported | KVM: read and write access to `/dev/kvm` |
| Linux | arm64 | Supported | KVM: read and write access to `/dev/kvm` |
| macOS | Intel | Not supported | |
| Windows | x86_64 | Through WSL2 with KVM, see [Getting started](README.md#verify-virtualization-support) | |

Check the host:

```bash
# Linux: the device must exist and your user must be able to open it
ls -l /dev/kvm
sudo usermod -aG kvm "$USER"   # then log out and in again

# macOS: must print arm64
uname -m
```

No daemon and no root: the engine runs inside the process that uses it. Boxes, images and logs live in
`~/.acryl/microvm` unless you set `ACRYLMICROVM_HOME` or pass `--home`.

## The command (`acrylmicrovm`)

### npx, pnpm dlx, yarn dlx, bunx: run without installing

```bash
npx acrylmicrovm run --rm alpine:latest echo hello
pnpm dlx acrylmicrovm run --rm alpine:latest echo hello
yarn dlx acrylmicrovm run --rm alpine:latest echo hello
bunx acrylmicrovm run --rm alpine:latest echo hello
```

The first call downloads the package for your platform (about 85 MB), later calls start in about a second.
Nothing has to compile: each platform package carries the finished binary.

### npm, pnpm, yarn, bun: install it

```bash
npm install -g acrylmicrovm      # then: acrylmicrovm --help
pnpm add -g acrylmicrovm
yarn global add acrylmicrovm     # yarn 1; with yarn 2+ use `yarn dlx`
bun add -g acrylmicrovm
```

Inside a project, add it as a dependency and run it through the package manager:

```bash
npm i acrylmicrovm   && npx acrylmicrovm --version
pnpm add acrylmicrovm && pnpm exec acrylmicrovm --version
yarn add acrylmicrovm && yarn acrylmicrovm --version
bun add acrylmicrovm  && bun x acrylmicrovm --version
```

### deno

```bash
deno run -A npm:acrylmicrovm run --rm alpine:latest echo hello
```

Deno does not install packages that are younger than 24 hours by default. Right after a release, add
`--minimum-dependency-age=0`, or wait a day.

### cargo binstall: the prebuilt binary through cargo

```bash
cargo binstall acrylmicrovm-cli
```

[cargo-binstall](https://github.com/cargo-bins/cargo-binstall) downloads the binary from the GitHub release in a few
seconds and installs it like `cargo install` would.

### cargo install: build it from crates.io

```bash
cargo install acrylmicrovm-cli
acrylmicrovm --version
```

Needs Rust 1.88 or later, a C compiler (`build-essential` on Debian and Ubuntu, Xcode command line tools on
macOS), `curl` and `tar`. It does not need protoc. The build takes about two minutes and downloads the prebuilt runtime (guest
agent, shim and guest kernel, about 30 MB) from the matching GitHub release, then embeds it in the binary.

### Install script

```bash
curl -fsSL https://github.com/acryldev/acrylmicrovm/releases/latest/download/install.sh | sh
```

Installs `acrylmicrovm` to `~/.local/bin` and checks its SHA-256. Choose another folder or version with
environment variables on the `sh` side of the pipe:

```bash
curl -fsSL https://github.com/acryldev/acrylmicrovm/releases/latest/download/install.sh \
  | ACRYLMICROVM_INSTALL_DIR=/usr/local/bin ACRYLMICROVM_VERSION=v0.13.1 sh
```

To check the script before running it, see [verifying downloads](../reference/cli/README.md#verifying-installsh-before-running-it).

### Download the binary

Each [release](https://github.com/acryldev/acrylmicrovm/releases) has the command as one self-contained file:

| Platform | File |
| --- | --- |
| macOS, Apple Silicon | `acrylmicrovm-cli-v<version>-aarch64-apple-darwin.tar.gz` |
| Linux x86_64 | `acrylmicrovm-cli-v<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux arm64 | `acrylmicrovm-cli-v<version>-aarch64-unknown-linux-gnu.tar.gz` |

```bash
tar xzf acrylmicrovm-cli-v0.13.1-x86_64-unknown-linux-gnu.tar.gz
./acrylmicrovm --version
```

## The Node.js SDK

```bash
npm install acrylmicrovm
pnpm add acrylmicrovm
yarn add acrylmicrovm
bun add acrylmicrovm
```

```javascript
import { SimpleBox } from 'acrylmicrovm';

const box = new SimpleBox({ image: 'alpine:latest' });
try {
  const result = await box.exec('echo', 'hello from a box');
  console.log(result.stdout);
} finally {
  await box.stop();
}
```

Runs with Node.js 18 or later, Bun and Deno (`import { SimpleBox } from 'npm:acrylmicrovm'`). The same package
also gives you the `acrylmicrovm` command. Next: [Node.js quick start](quickstart-nodejs.md).

## The Rust library

```bash
cargo add acrylmicrovm tokio futures --features tokio/macros,tokio/rt-multi-thread
```

The first build downloads the prebuilt runtime from the GitHub release, like `cargo install` above. Next:
[Rust quick start](quickstart-rust.md).

## Python, C and Go

The Python package is not on PyPI yet, and the C and Go SDKs are not published to a registry. Build them from a
checkout as described in [Python](quickstart-python.md), [C](quickstart-c.md) and
[Go](../../sdks/go/README.md).

## From source

```bash
git clone --recurse-submodules https://github.com/acryldev/acrylmicrovm.git
cd acrylmicrovm
make setup
make cli:release      # target/release/acrylmicrovm
```

See [Building](../contributing/development/building.md) and [Dependency pins](../contributing/development/dependency-pins.md).

## Check the install

```bash
acrylmicrovm --version
acrylmicrovm run --rm alpine:latest uname -r    # prints the box's own kernel, for example 6.12.87
```

## Troubleshooting

**`The package acrylmicrovm-<platform> is missing`.** The platform package was skipped. Reinstall without
`--omit=optional` / `--no-optional`. Right after a new release the registry can take a few minutes to serve the
platform package; clear the npx cache (`rm -rf ~/.npm/_npx`) and try again.

**`acrylmicrovm has no prebuilt binary for <platform>`.** Your OS or CPU is not one of the supported platforms
above. On an Intel Mac there is no supported way; on Windows use WSL2.

**`/dev/kvm` permission denied.** Add your user to the `kvm` group and log in again (on WSL2 run
`wsl.exe --shutdown` first).

**Linux: the sandbox cannot create namespaces.** Ubuntu 24.04 blocks unprivileged user namespaces by default; see
[Dependency pins: host requirements](../contributing/development/dependency-pins.md#host-requirements-to-run-a-box).

**`cargo install` fails to download the runtime.** The crate fetches it from
`github.com/acryldev/acrylmicrovm/releases`. Behind a proxy, download the `acrylmicrovm-runtime-v<version>-<target>.tar.gz`
file yourself and point `ACRYLMICROVM_RUNTIME_URL` at it (a `file://` or `https://` URL).

More in [Troubleshooting](../guides/troubleshooting.md).
