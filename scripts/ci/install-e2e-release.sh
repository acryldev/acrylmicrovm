#!/usr/bin/env bash
# Modified by ACRYL, 2026
# Install release clients at the paths used by the cloud acceptance suite.
set -euo pipefail

component=${1:?usage: install-e2e-release.sh validate|python|node|cli|c}
tag=${RELEASE_TAG:-}
if [[ ! "$tag" =~ ^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
    echo 'RELEASE_TAG must be a stable vX.Y.Z tag' >&2
    exit 2
fi
case "$component" in
    validate|python|node|cli|c) ;;
    *) echo "unknown release component: $component" >&2; exit 2 ;;
esac

if [[ "$component" == validate ]]; then
    published=$(gh release view "$tag" --repo acryldev/acrylmicrovm \
        --json tagName,isDraft --jq 'select(.isDraft == false) | .tagName')
    [[ "$published" == "$tag" ]] || { echo "release $tag is not published" >&2; exit 1; }
    exit 0
fi

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
scratch=$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/e2e-release.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
version=${tag#v}
download() {
    gh release download "$tag" --repo acryldev/acrylmicrovm --pattern "$1" --dir "$scratch"
}

case "$component" in
    python)
        abi=$(python3 -c 'import sys; print(f"cp{sys.version_info.major}{sys.version_info.minor}")')
        wheel="acrylmicrovm-${version}-${abi}-${abi}-manylinux_2_28_x86_64.whl"
        download "$wheel"
        pip install --break-system-packages --only-binary=:all: --force-reinstall "$scratch/$wheel"
        ;;
    node)
        package="acryldev-acrylmicrovm-${version}.tgz"
        platform="acryldev-acrylmicrovm-linux-x64-gnu-${version}.tgz"
        download "$package"
        download "$platform"
        mkdir -p "$root/sdks/node/native"
        tar xzf "$scratch/$package" --strip-components=1 -C "$root/sdks/node"
        tar xzf "$scratch/$platform" --strip-components=1 -C "$root/sdks/node/native" \
            package/acrylmicrovm.linux-x64-gnu.node
        (
            cd "$root/sdks/node"
            # The release carries the SDK; these tools run the checkout's test drivers.
            # SDK build metadata can break npm's peer resolution even with --omit=dev.
            # Keep runtime dependencies and the downloaded addon; install only test tools.
            npm pkg delete optionalDependencies devDependencies workspaces --workspaces=false
            npm install --ignore-scripts --workspaces=false --omit=dev --save-prod \
                --package-lock=false playwright-core@1.58.0 tsx
            npx --no-install tsx --eval 'const ready: boolean = true; if (!ready) throw Error("tsx unavailable")'
            node --input-type=module -e \
                'import "playwright-core"; import { JsAcrylMicrovm } from "./dist/index.js"; if (typeof JsAcrylMicrovm.rest !== "function") throw Error("release REST SDK missing")'
        )
        ;;
    cli)
        download install.sh
        install_dir="$root/target/e2e-bin"
        ACRYLMICROVM_VERSION="$tag" ACRYLMICROVM_INSTALL_DIR="$install_dir" sh "$scratch/install.sh"
        "$install_dir/acrylmicrovm" --version
        echo "$install_dir" >> "${GITHUB_PATH:?GITHUB_PATH is required}"
        ;;
    c)
        archive="acrylmicrovm-c-${tag}-linux-x64-gnu.tar.gz"
        download "$archive"
        mkdir -p "$root/target/e2e-c-sdk"
        tar xzf "$scratch/$archive" --strip-components=1 -C "$root/target/e2e-c-sdk"
        ;;
esac
