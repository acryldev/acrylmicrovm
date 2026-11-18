# Rust quick start

Get up and running with ACRYL microVMs Rust crate in 5 minutes.

## Installation

**Requirements:** Rust 1.88 or later

Add ACRYL microVMs to your project:

```bash
cargo new hello && cd hello
cargo add acrylmicrovm tokio futures --features tokio/macros,tokio/rt-multi-thread
```

The first build downloads the prebuilt runtime (guest agent, shim, guest kernel) from the matching GitHub release and
embeds it in your binary; it needs a C compiler, `curl` and `tar`, but not protoc. For the command line instead, see
[Installation](installation.md).

## Basic execution

Create a file `src/main.rs`:

```rust
use acrylmicrovm::{AcrylMicrovmRuntime, BoxOptions, BoxCommand, RootfsSpec};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create runtime
    let runtime = AcrylMicrovmRuntime::default_runtime();

    // Create box
    let options = BoxOptions {
        rootfs: RootfsSpec::Image("alpine:latest".into()),
        ..Default::default()
    };
    let litebox = runtime.create(options, None).await?;

    // Execute command
    let mut execution = litebox
        .exec(BoxCommand::new("echo").arg("Hello from AcrylMicrovm!"))
        .await?;

    // Stream stdout
    let mut stdout = execution.stdout().unwrap();
    while let Some(line) = stdout.next().await {
        println!("{}", line);
    }

    Ok(())
}
```

Run it:
```bash
cargo run
```

## From source (development)

For contributing or local development:

```bash
# Clone repository
git clone https://github.com/acryldev/acrylmicrovm.git
cd acrylmicrovm

# Initialize submodules (critical!)
git submodule update --init --recursive

# Install platform dependencies
make setup

# Build
cargo build

# Run tests
cargo test
```

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for detailed build instructions.

## Next steps

- **[Concepts](../concepts/README.md)** - Understand how ACRYL microVMs works
  - Core components (Runtime, LiteBox, VMM, Portal)
  - Image management and rootfs preparation
  - Host-guest communication protocol
- **[Reference](../reference/README.md#rust-api)** - Rust API reference
