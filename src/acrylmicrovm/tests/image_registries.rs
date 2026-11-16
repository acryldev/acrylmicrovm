// Modified by ACRYL, 2026
//! Integration tests for structured image registry configuration.

mod common;

use acrylmicrovm::runtime::options::AcrylMicrovmOptions;
use acrylmicrovm::{AcrylMicrovmRuntime, ImageRegistry};
use common::home::PerTestBoxHome;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn structured_image_registries_pull_unqualified_image() {
    let home = PerTestBoxHome::isolated_in("/tmp");
    let runtime = AcrylMicrovmRuntime::new(AcrylMicrovmOptions {
        home_dir: home.path.clone(),
        image_registries: common::test_registries(),
        ..Default::default()
    })
    .unwrap();

    let images = runtime.images().unwrap();
    let result = images.pull("alpine:latest").await.unwrap();
    let list = images.list().await.unwrap();

    assert_eq!(result.reference(), "alpine:latest");
    assert!(result.config_digest().starts_with("sha256:"));
    assert!(result.layer_count() > 0);
    assert!(
        list.iter()
            .any(|image| image.repository.contains("alpine") && image.tag == "latest")
    );

    runtime
        .shutdown(Some(common::TEST_SHUTDOWN_TIMEOUT))
        .await
        .unwrap();
}

#[test]
fn structured_image_registries_validate_at_runtime_start() {
    let home = PerTestBoxHome::isolated_in("/tmp");
    let result = AcrylMicrovmRuntime::new(AcrylMicrovmOptions {
        home_dir: home.path.clone(),
        image_registries: vec![ImageRegistry::https("https://registry.local")],
        ..Default::default()
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("host[:port]"));
}
