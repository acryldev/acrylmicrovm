// Modified by ACRYL, 2026
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn custom_kernel_requires_explicit_rc_opt_in() {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("acrylmicrovm"));
    command
        .env_remove("ACRYLMICROVM_EXPERIMENTAL")
        .args([
            "--url",
            "http://127.0.0.1:1",
            "create",
            "--rootfs",
            "/definitely/missing/rootfs",
            "--kernel",
            "/definitely/missing/vmlinux",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "ACRYLMICROVM_EXPERIMENTAL=custom-kernel",
        ));
}

#[test]
fn unknown_rc_feature_fails_before_runtime_setup() {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("acrylmicrovm"));
    command
        .env("ACRYLMICROVM_EXPERIMENTAL", "custom-kernal")
        .args([
            "--url",
            "http://127.0.0.1:1",
            "create",
            "--rootfs",
            "/definitely/missing/rootfs",
        ])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("unknown feature 'custom-kernal'")
                .and(predicate::str::contains("ACRYLMICROVM_EXPERIMENTAL")),
        );
}
