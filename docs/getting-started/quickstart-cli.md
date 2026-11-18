# Command line quick start

Run code in a micro-VM from your terminal in five minutes. Install the command first
([Installation](installation.md)), or put `npx` in front of every command below.

## One command, one box

```bash
acrylmicrovm run --rm alpine:latest echo hello
```

`run` pulls the image (first time only), starts a micro-VM with its own kernel, runs the command and prints its
output. `--rm` removes the box when the command ends. The exit code of the command is the exit code of
`acrylmicrovm`:

```bash
acrylmicrovm run --rm alpine:latest sh -c 'exit 5'; echo $?   # 5
acrylmicrovm run --rm alpine:latest uname -r                    # the box's kernel, not the host's
```

Useful options:

| Option | Meaning |
| --- | --- |
| `-e NAME=value` | Set an environment variable in the box |
| `-w /path` | Working directory in the box |
| `--cpus 2`, `--memory 512` | CPUs and memory (MiB) for this box |
| `-v /host/path:/box/path` | Share a host folder with the box |
| `-p 8080:80` | Publish a box port on the host |
| `-i`, `-t` | Keep stdin open, allocate a terminal (`-it` for an interactive shell) |
| `-d` | Start in the background |
| `--name demo` | Give the box a name |

```bash
acrylmicrovm run --rm -e GREETING=hi --cpus 2 --memory 512 alpine:latest \
  sh -c 'echo $GREETING, $(nproc) CPUs'
acrylmicrovm run --rm -it alpine:latest sh        # an interactive shell inside the box
printf 'echo from stdin\n' | acrylmicrovm run --rm -i alpine:latest sh
```

## A box that stays

A box keeps its disk until you remove it, so you can prepare it once and use it many times.

```bash
acrylmicrovm create --name demo node:slim
acrylmicrovm start demo
acrylmicrovm exec demo -- node -v
```

Copy files in and out:

```bash
echo 'console.log("hello from a file")' > hello.js
acrylmicrovm cp hello.js demo:/root/hello.js
acrylmicrovm exec demo -- node /root/hello.js

acrylmicrovm exec demo -- sh -c 'echo made in the box > /root/out.txt'
acrylmicrovm cp demo:/root/out.txt ./out.txt
```

Stop and start it again; the files are still there:

```bash
acrylmicrovm stop demo
acrylmicrovm start demo
acrylmicrovm exec demo -- cat /root/out.txt
```

See what exists and clean up:

```bash
acrylmicrovm list -a         # all boxes, running or stopped
acrylmicrovm images          # pulled images
acrylmicrovm rm -f demo      # stop if needed, then remove
```

## Run it as a service

`serve` starts a REST server with the same operations, for other machines or other languages:

```bash
ACRYLMICROVM_SERVE_API_KEY=change-me acrylmicrovm serve --port 8100
curl -H 'authorization: Bearer change-me' http://127.0.0.1:8100/v1/boxes
```

Point the command at it with `--url` (or `acrylmicrovm auth login`). The API is described in the
[CLI reference](../reference/cli/README.md) and the [hosted API notes](../reference/hosted-api.md).

## Where things live

Boxes, images and logs are in `~/.acryl/microvm`. Use another place with `--home /path` or
`ACRYLMICROVM_HOME=/path`, for example one folder per project or per test run.

## Next

- [Installation](installation.md): every way to install
- [Node.js](quickstart-nodejs.md) and [Rust](quickstart-rust.md) quick starts: boxes from code
- [CLI reference](../reference/cli/README.md): every command and option
- [Concepts](../concepts/README.md): images, storage, networking, isolation
