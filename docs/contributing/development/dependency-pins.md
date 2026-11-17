# Dependency pins and a Linux build recipe

The native runtime builds four third-party repositories from source or from a prebuilt release. The pins below are
the commits the 0.13.1 release was built with. They are git submodules pointing at the upstream repositories, so
`git submodule update --init` fetches them.

| Path | Pinned commit |
| --- | --- |
| `src/deps/libkrun-sys/vendor/libkrun` | `e12b9b3780ffa8df9f3e1797b217d13453479167` |
| `src/deps/libkrun-sys/vendor/libkrunfw` | `e0647fa7a3932c4570fb2c0fd483b9f7a382ebdd` |
| `src/deps/e2fsprogs-sys/vendor/e2fsprogs` | `da631e117dcf8797bfda0f48bdaa05ac0fbcf7af` |
| `src/deps/bubblewrap-sys/vendor/bubblewrap` | `9ca3b05ec787acfb4b17bed37db5719fa777834f` |

`libkrun-sys/build.rs` also downloads the prebuilt kernel library of the upstream libkrunfw v5.4.0 release from
`github.com/libkrun/libkrunfw/releases`, checked against a pinned SHA-256. Licences of these parts are listed in
`NOTICE`. They stay until the VMM in `src/vmm` and its own guest kernel replace libkrun and libkrunfw.

## Verified build, 2026-11-16

Ubuntu 24.04 x86_64 in a container (root inside; packages: build-essential, musl-tools, libclang-dev, llvm, flex,
bison, bc, libelf-dev, meson, ninja-build, libcap-dev, protobuf-compiler, cmake, gperf, patchelf; Go 1.24.12; stable
Rust with the `x86_64-unknown-linux-musl` target), then `make cli:release`. It produced `acrylmicrovm 0.13.1`
(115 MB) in about 7 minutes on a 16-core machine.

## Host requirements to run a box

- `/dev/kvm` readable and writable by the user.
- The host sandbox needs unprivileged user namespaces. Ubuntu 24.04 blocks them by default
  (`kernel.apparmor_restrict_unprivileged_userns=1`); an administrator has to install the `bubblewrap` AppArmor
  profile (`sudo apt install --reinstall bubblewrap`) or relax that setting. For debugging, `--security=disable` on
  `run` skips the sandbox; `serve` has no such switch.
- In a Docker container the sandbox also needs `--device /dev/kvm --cap-add SYS_ADMIN --security-opt
  apparmor=unconfined --security-opt seccomp=unconfined --security-opt systempaths=unconfined`.
