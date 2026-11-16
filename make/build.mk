# Modified by ACRYL, 2026

PHONY_TARGETS += guest shim runtime vmm vmm\:boot vmm\:boot\:host cli cli\:release skillbox-image

vmm:
	@cargo build -p acrylmicrovm-hypervisor -p acrylmicrovm-vmm

# Keep the test init separate from guest binaries assembled into product runtimes.
PHONY_TARGETS += vmm\:guest-mount-init
vmm\:guest-mount-init:
	@bash -ec 'source "$(SCRIPT_DIR)/build/build-libseccomp.sh"; \
	ensure_libseccomp_for_target x86_64-unknown-linux-musl; \
	cargo build -p acrylmicrovm-guest --bin acrylmicrovm-guest --features integration-test-init --target x86_64-unknown-linux-musl --release --target-dir "$(PROJECT_ROOT)/target/vmm/guest-mount"'

vmm\:boot: export _ACRYLMICROVM_BOOT_OUTPUT_ARG := $(value BOOT_OUTPUT)
vmm\:boot: export _ACRYLMICROVM_BOOT_JOBS_ARG := $(value BOOT_JOBS)
vmm\:boot:
	@bash "$(SCRIPT_DIR)/build/build-vmm-boot-in-box.sh"

vmm\:boot\:host: export _ACRYLMICROVM_BOOT_OUTPUT_ARG := $(value BOOT_OUTPUT)
vmm\:boot\:host: export _ACRYLMICROVM_BOOT_JOBS_ARG := $(value BOOT_JOBS)
vmm\:boot\:host:
	@bash "$(SCRIPT_DIR)/build/build-vmm-boot.sh"

guest: export _ACRYLMICROVM_GUEST_TARGET_ARG := $(value GUEST_TARGET)
guest: export _ACRYLMICROVM_PROFILE_ARG := $(value PROFILE)
guest:
	@target="$${_ACRYLMICROVM_GUEST_TARGET_ARG:-$$(bash "$$PWD/scripts/util.sh" --target)}"; \
	profile="$${_ACRYLMICROVM_PROFILE_ARG:-release}"; \
	bash "$$PWD/scripts/build/build-guest.sh" --target "$$target" --profile "$$profile" && \
	bash "$$PWD/scripts/build/build-guest-deps.sh" --target "$$target" --profile "$$profile"

shim:
	@bash $(SCRIPT_DIR)/build/build-shim.sh

runtime:
	@bash $(SCRIPT_DIR)/build/build-runtime.sh --profile release

runtime\:debug:
	@bash $(SCRIPT_DIR)/build/build-runtime.sh --profile debug

cli: runtime\:debug
	@echo "🔨 Building acrylmicrovm CLI..."
	@cargo build -p acrylmicrovm-cli
	@echo "✅ CLI built: ./target/debug/acrylmicrovm"

cli\:release: runtime
	@echo "🔨 Building acrylmicrovm CLI (release)..."
	@cargo build -p acrylmicrovm-cli --release
	@echo "✅ CLI built: ./target/release/acrylmicrovm"

# Build SkillBox container image (all-in-one AI CLI with noVNC)
# Usage: make skillbox-image [APT_SOURCE=mirrors.aliyun.com]
skillbox-image:
	@echo "🐳 Building SkillBox container image..."
	@docker build $(if $(APT_SOURCE),--build-arg APT_SOURCE=$(APT_SOURCE)) -t acrylmicrovm-skillbox:latest src/acrylmicrovm/resources/images/skillbox/
	@echo "✅ SkillBox image built: acrylmicrovm-skillbox:latest"
