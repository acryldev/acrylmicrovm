#!/bin/bash
# Modified by ACRYL, 2026
# Setup script for ACRYL microVMs test/dev-only tooling.
#
# Intended to be run *after* build deps are installed (e.g. after `make setup:build`).
# Idempotent.

set -e

SETUP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT_DIR="$(cd "$SETUP_DIR/.." && pwd)"

source "$SCRIPT_DIR/common.sh"
source "$SETUP_DIR/setup-common.sh"

main() {
    print_header "AcrylMicrovm Test/Dev Setup"

    # Ensure we run dev extras even if caller exported ACRYLMICROVM_SETUP_MODE=build.
    export ACRYLMICROVM_SETUP_MODE=dev

    run_dev_extras

    print_header "Setup Complete"
}

main "$@"
