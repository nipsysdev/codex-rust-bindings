//! Integration tests for node metrics and Mix private queries.

use storage_bindings::{StorageConfig, StorageNode};
use tempfile::tempdir;

/// Mix relay pool + proxies fixture (a working default configuration).
const MIX_CONFIG: &str = include_str!("data/mix_config.json");

fn base_config(dir: &std::path::Path, disc_port: u16) -> StorageConfig {
    StorageConfig::new()
        .data_dir(dir.join("storage_data"))
        .nat("extip:127.0.0.1")
        .discovery_port(disc_port)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_metrics() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::try_init();
    let temp_dir = tempdir()?;

    let node = StorageNode::new(base_config(temp_dir.path(), 8110)).await?;
    node.start().await?;

    let metrics = node.get_metrics().await?;
    assert!(!metrics.is_empty(), "metrics should not be empty");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_toggle_private_queries_disable_without_mix() -> Result<(), Box<dyn std::error::Error>>
{
    let _ = env_logger::try_init();
    let temp_dir = tempdir()?;

    let node = StorageNode::new(base_config(temp_dir.path(), 8111)).await?;
    node.start().await?;

    let previous = node.toggle_private_queries(false).await?;
    assert!(!previous, "private queries should be disabled by default");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_toggle_private_queries_enable_without_mix_fails(
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::try_init();
    let temp_dir = tempdir()?;

    let node = StorageNode::new(base_config(temp_dir.path(), 8112)).await?;
    node.start().await?;

    assert!(
        node.toggle_private_queries(true).await.is_err(),
        "enabling private queries without Mix configured should fail"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_toggle_private_queries_with_mix() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::try_init();
    let temp_dir = tempdir()?;

    let mut config = StorageConfig::from_json(MIX_CONFIG)?;
    config.data_dir = Some(temp_dir.path().join("storage_data"));
    config.nat = Some("extip:127.0.0.1".to_string());
    config.discovery_port = Some(8113);

    let node = StorageNode::new(config).await?;
    node.start().await?;

    // Mix auto-enables private queries at startup, so disabling returns true.
    let previous = node.toggle_private_queries(false).await?;
    assert!(
        previous,
        "private queries should be enabled at startup when Mix is configured"
    );

    // Re-enabling is allowed because Mix is configured.
    let previous = node.toggle_private_queries(true).await?;
    assert!(
        !previous,
        "previous state should be disabled after toggling off"
    );

    Ok(())
}
