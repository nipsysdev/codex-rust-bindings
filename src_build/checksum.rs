use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;

/// Calculates SHA256 checksum of a file
pub fn calculate_sha256(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    println!(
        "  [CHECKSUM] Calculating SHA256 for: {}",
        file_path.display()
    );

    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    let mut total_bytes = 0u64;

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        total_bytes += n as u64;
    }

    let checksum = hex::encode(hasher.finalize());
    println!("  [CHECKSUM] ✓ SHA256 calculated: {}", checksum);
    println!("  [CHECKSUM] ✓ Total bytes processed: {}", total_bytes);

    Ok(checksum)
}

/// Verifies the checksum of an archive file against an expected checksum
pub fn verify_archive_checksum(
    archive_path: &Path,
    expected_checksum: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  [CHECKSUM] Verifying archive checksum...");
    println!("  [CHECKSUM] Archive path: {}", archive_path.display());
    println!("  [CHECKSUM] Expected checksum: {}", expected_checksum);

    let actual_checksum = calculate_sha256(archive_path)?;

    if actual_checksum != expected_checksum {
        println!("  [CHECKSUM] ✗ Archive checksum mismatch!");
        println!("  [CHECKSUM]   Expected: {}", expected_checksum);
        println!("  [CHECKSUM]   Actual:   {}", actual_checksum);
        return Err(format!(
            "Archive checksum mismatch:\n  Expected: {}\n  Actual: {}",
            expected_checksum, actual_checksum
        )
        .into());
    }

    println!("  [CHECKSUM] ✓ Archive checksum verified");
    Ok(())
}
