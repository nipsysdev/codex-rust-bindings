//! Thread-safe tests for CodexNode
//!
//! These tests verify that CodexNode can be safely used across threads
//! and implements the required traits for concurrent operations.

use codex_bindings::{CodexConfig, CodexNode};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_thread_safe_node_creation() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    let node = CodexNode::new(config).unwrap();
    assert!(!node.is_started());
}

#[tokio::test]
async fn test_thread_safe_node_lifecycle() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    let mut node = CodexNode::new(config).unwrap();

    // Start the node
    node.start().unwrap();
    assert!(node.is_started());

    // Get some info
    let version = node.version().unwrap();
    assert!(!version.is_empty());

    let peer_id = node.peer_id().unwrap();
    assert!(!peer_id.is_empty());

    // Stop the node
    node.stop().unwrap();
    assert!(!node.is_started());
}

#[tokio::test]
async fn test_node_cloning() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    let mut node1 = CodexNode::new(config).unwrap();
    let node2 = node1.clone();

    // Both should reference the same underlying node
    assert!(!node1.is_started());
    assert!(!node2.is_started());

    // Start through one reference
    node1.start().unwrap();

    // Both should show as started
    assert!(node1.is_started());
    assert!(node2.is_started());
}

#[tokio::test]
async fn test_concurrent_access() {
    use tokio::task::JoinSet;

    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    let node = Arc::new(CodexNode::new(config).unwrap());
    node.start_async().await.unwrap();

    let mut set = JoinSet::new();

    // Spawn multiple concurrent operations
    for _ in 0..5 {
        let node_clone = node.clone();
        set.spawn(async move {
            let version = node_clone.version().unwrap();
            assert!(!version.is_empty());
        });
    }

    // Wait for all to complete
    while let Some(result) = set.join_next().await {
        result.unwrap();
    }
}

#[test]
fn test_send_sync_traits() {
    // This test verifies that CodexNode implements Send and Sync
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());
    let _node = CodexNode::new(config).unwrap();

    assert_send::<CodexNode>();
    assert_sync::<CodexNode>();

    // Test that Arc<CodexNode> is Send (needed for sharing across threads)
    assert_send::<Arc<CodexNode>>();
}

#[test]
fn test_clone_trait() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    let mut node1 = CodexNode::new(config).unwrap();
    let node2 = node1.clone();

    // Both should be valid
    assert!(!node1.is_started());
    assert!(!node2.is_started());

    // Verify they share the same underlying state
    node1.start().unwrap();
    assert!(node1.is_started());
    assert!(node2.is_started());
}

#[tokio::test]
async fn test_send_between_threads() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());
    let node = CodexNode::new(config).unwrap();

    // Test that node can be sent to another thread
    let result = tokio::task::spawn(async move {
        // Use node in a different thread
        let _version = node.version().unwrap();
        "success"
    })
    .await;

    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_async_file_upload() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());
    let node = Arc::new(CodexNode::new(config).unwrap());

    // Start the node
    node.start_async().await.unwrap();

    // Create a test file
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, b"Hello, Codex!").unwrap();

    // Try to use upload_file in an async context
    let options = codex_bindings::UploadOptions::new().filepath(&file_path);

    // This should work because CodexNode is Send and can be shared across threads
    let result = codex_bindings::upload_file(&node, options).await;

    assert!(result.is_ok(), "Upload should succeed");

    // Stop the node
    node.stop_async().await.unwrap();
}

#[tokio::test]
async fn test_multiple_concurrent_operations() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());
    let node = Arc::new(CodexNode::new(config).unwrap());

    // Start the node
    node.start_async().await.unwrap();

    // Perform multiple operations concurrently
    let mut handles = Vec::new();

    for i in 0..5 {
        let node_clone = node.clone();
        let handle = tokio::task::spawn(async move {
            // Multiple threads accessing the C library are properly synchronized
            let version = node_clone.version().unwrap();
            let peer_id = node_clone.peer_id().unwrap();
            (i, version, peer_id)
        });
        handles.push(handle);
    }

    // Wait for all operations
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    // All operations should complete successfully
    assert_eq!(
        results.len(),
        5,
        "All concurrent operations should complete"
    );

    // Stop the node
    node.stop_async().await.unwrap();
}

#[tokio::test]
async fn test_shared_node_across_tasks() {
    let temp_dir = tempdir().unwrap();
    let config = CodexConfig::new().data_dir(temp_dir.path());

    // Simulate application state with shared node
    struct AppState {
        node: Arc<CodexNode>,
    }

    let state = AppState {
        node: Arc::new(CodexNode::new(config).unwrap()),
    };

    // Simulate multiple concurrent tasks
    let mut handles = Vec::new();

    // Task 1: Get node info
    let node_clone = state.node.clone();
    handles.push(tokio::task::spawn(async move {
        let version = node_clone.version().unwrap();
        format!("Node version: {}", version)
    }));

    // Task 2: Get peer ID
    let node_clone = state.node.clone();
    handles.push(tokio::task::spawn(async move {
        let peer_id = node_clone.peer_id().unwrap();
        format!("Peer ID: {}", peer_id)
    }));

    // Task 3: Create and start a new node
    handles.push(tokio::task::spawn(async move {
        // Use spawn_blocking for methods that need &mut self
        tokio::task::spawn_blocking(move || {
            let mut node = CodexNode::new(CodexConfig::new()).unwrap();
            node.start().unwrap();
            node
        })
        .await
        .unwrap();
        "Node started".to_string()
    }));

    // Wait for all tasks
    for handle in handles {
        let result = handle.await.unwrap();
        println!("Task result: {}", result);
    }
}
