//! Integration tests for Mix private queries.

use storage_bindings::{StorageConfig, StorageNode};
use tempfile::tempdir;

/// An isolated node: no bootstrap peers, which libstorage only allows with an external IP.
fn base_config(dir: &std::path::Path, disc_port: u16) -> StorageConfig {
    StorageConfig::new()
        .data_dir(dir.join("storage_data"))
        .nat("extip:127.0.0.1")
        .no_bootstrap_node(true)
        .discovery_port(disc_port)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_toggle_private_queries_disable_without_mix() -> Result<(), Box<dyn std::error::Error>>
{
    let _ = env_logger::try_init();
    let temp_dir = tempdir()?;

    let node = StorageNode::new(base_config(temp_dir.path(), 8110)).await?;
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

    let node = StorageNode::new(base_config(temp_dir.path(), 8111)).await?;
    node.start().await?;

    assert!(
        node.toggle_private_queries(true).await.is_err(),
        "enabling private queries without Mix configured should fail"
    );

    Ok(())
}
