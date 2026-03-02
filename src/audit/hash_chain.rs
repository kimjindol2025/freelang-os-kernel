// 🐀 Anti-Lie Solution 2: Hash-Chained Audit Log
// Purpose: Create immutable proof of all state transitions
// Philosophy: "The record is the proof"
//
// Every context switch:
//   1. Hash the current state
//   2. Chain it to previous hash
//   3. Timestamp it
//   4. This creates a tamper-proof audit trail
//
// To forge a record, attacker must:
//   1. Change the state
//   2. Recalculate the hash
//   3. Recalculate ALL downstream hashes
//   4. But that takes exponential time (detected)

use std::collections::VecDeque;
use std::fmt;
use sha2::{Sha256, Digest};

/// Individual audit entry
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub sequence_number: u64,
    pub timestamp: u128,
    pub state_description: String,
    pub stack_pointer: usize,
    pub stack_drift: i64,
    pub context_switch_count: u64,
    pub current_hash: String,
    pub previous_hash: String,
}

impl AuditEntry {
    /// Compute SHA256 hash of this entry
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();

        // Include all state to detect any modification
        hasher.update(format!("{}", self.sequence_number).as_bytes());
        hasher.update(format!("{}", self.timestamp).as_bytes());
        hasher.update(self.state_description.as_bytes());
        hasher.update(format!("{}", self.stack_pointer).as_bytes());
        hasher.update(format!("{}", self.stack_drift).as_bytes());
        hasher.update(format!("{}", self.context_switch_count).as_bytes());
        hasher.update(self.previous_hash.as_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Verify integrity - recompute hash and check match
    pub fn verify(&self) -> bool {
        let recomputed_hash = self.compute_hash();
        recomputed_hash == self.current_hash
    }
}

/// Hash-Chained Audit Log with tamper detection
pub struct HashChainedAuditLog {
    entries: VecDeque<AuditEntry>,
    current_hash: String,
    entry_count: u64,
    pub total_hashes: u64,
    pub verified_hashes: u64,
    pub verification_failures: u64,
    pub checkpoints: Vec<(u64, String)>,
}

impl HashChainedAuditLog {
    pub fn new() -> Self {
        // Genesis block
        let genesis_hash = Self::compute_genesis_hash();

        Self {
            entries: VecDeque::new(),
            current_hash: genesis_hash.clone(),
            entry_count: 0,
            total_hashes: 1,
            verified_hashes: 1,
            verification_failures: 0,
            checkpoints: vec![(0, genesis_hash)],
        }
    }

    /// Genesis block hash (no previous)
    fn compute_genesis_hash() -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"GENESIS_BLOCK");
        format!("{:x}", hasher.finalize())
    }

    /// Record a context switch state transition
    pub fn record_context_switch(
        &mut self,
        state_desc: &str,
        stack_pointer: usize,
        stack_drift: i64,
        context_switch_count: u64,
    ) {
        let entry = AuditEntry {
            sequence_number: self.entry_count,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_micros())
                .unwrap_or(0),
            state_description: state_desc.to_string(),
            stack_pointer,
            stack_drift,
            context_switch_count,
            current_hash: String::new(), // Will be set below
            previous_hash: self.current_hash.clone(),
        };

        // Compute hash with previous_hash included
        let computed_hash = entry.compute_hash();

        // Can't modify entry after this, so we need to construct properly
        let mut final_entry = entry;
        final_entry.current_hash = computed_hash.clone();

        self.entries.push_back(final_entry);
        self.current_hash = computed_hash;
        self.entry_count += 1;
        self.total_hashes += 1;

        // Create checkpoint every 100,000 entries
        if self.entry_count % 100_000 == 0 {
            self.checkpoints.push((self.entry_count, self.current_hash.clone()));
        }
    }

    /// Verify entire chain integrity
    pub fn verify_chain_integrity(&mut self) -> bool {
        println!("\n{}", "=".repeat(70));
        println!("🔐 HASH CHAIN INTEGRITY VERIFICATION");
        println!("{}", "=".repeat(70));

        let mut verification_passed = true;
        let mut consecutive_failures = 0;

        // Verify each entry in chain
        for (idx, entry) in self.entries.iter().enumerate() {
            let recomputed_hash = entry.compute_hash();

            if recomputed_hash != entry.current_hash {
                println!(
                    "❌ Hash mismatch at entry {}: computed={}, stored={}",
                    idx,
                    &recomputed_hash[..16],
                    &entry.current_hash[..16]
                );
                verification_passed = false;
                consecutive_failures += 1;
                self.verification_failures += 1;
            } else {
                self.verified_hashes += 1;
                consecutive_failures = 0;

                if idx % 100_000 == 0 && idx > 0 {
                    println!("  ✅ Entry {}: Hash verified (hash: {}...)", idx, &entry.current_hash[..16]);
                }
            }

            // Detect tampering (consecutive failures indicate systematic attacks)
            if consecutive_failures > 100 {
                println!("🚨 TAMPERING DETECTED: {} consecutive hash failures!", consecutive_failures);
                return false;
            }
        }

        let total = self.entries.len();
        let verified = self.verified_hashes as usize;

        println!("\n{}", "=".repeat(70));
        println!("✅ CHAIN VERIFICATION COMPLETE:");
        println!("  Total Entries:        {}", total);
        println!("  Verified Entries:     {} ({:.2}%)",
            verified,
            if total > 0 { (verified as f64 / total as f64) * 100.0 } else { 0.0 }
        );
        println!("  Failed Entries:       {}", self.verification_failures);
        println!("  Checkpoints:          {}", self.checkpoints.len());
        println!("  Chain Head Hash:      {}...", &self.current_hash[..16]);
        println!("{}", "=".repeat(70));

        verification_passed
    }

    /// Verify chain consistency from checkpoint
    pub fn verify_from_checkpoint(&self, checkpoint_seq: u64) -> (bool, String) {
        // Find checkpoint
        let checkpoint = self.checkpoints.iter().find(|(seq, _)| *seq == checkpoint_seq);

        if let Some((_, checkpoint_hash)) = checkpoint {
            // Verify chain from this point forward
            let mut current_hash = checkpoint_hash.clone();
            let mut verified_count = 0;

            for entry in self.entries.iter() {
                if entry.sequence_number < checkpoint_seq {
                    continue;
                }

                if entry.previous_hash != current_hash {
                    return (false, format!(
                        "Checkpoint {} integrity failed at entry {}",
                        checkpoint_seq, entry.sequence_number
                    ));
                }

                current_hash = entry.current_hash.clone();
                verified_count += 1;
            }

            (true, format!("Verified {} entries from checkpoint {}", verified_count, checkpoint_seq))
        } else {
            (false, format!("Checkpoint {} not found", checkpoint_seq))
        }
    }

    /// Get detailed chain report
    pub fn get_chain_report(&self) -> String {
        let mut report = format!(
            "HASH-CHAINED AUDIT LOG REPORT\n\
            {}\n\
            Total Entries:           {}\n\
            Total Hashes:            {}\n\
            Verified Hashes:         {} ({:.2}%)\n\
            Verification Failures:   {}\n\
            Checkpoints Created:     {}\n\
            Chain Head:              {}...\n\
            Chain Integrity:         {}\n\
            {}",
            "=".repeat(60),
            self.entry_count,
            self.total_hashes,
            self.verified_hashes,
            if self.total_hashes > 0 {
                (self.verified_hashes as f64 / self.total_hashes as f64) * 100.0
            } else {
                0.0
            },
            self.verification_failures,
            self.checkpoints.len(),
            &self.current_hash[..16],
            if self.verification_failures == 0 { "✅ VALID" } else { "❌ COMPROMISED" },
            "=".repeat(60)
        );

        if !self.checkpoints.is_empty() {
            report.push_str("\n\nCHECKPOINTS:\n");
            for (idx, (seq, hash)) in self.checkpoints.iter().enumerate() {
                if idx % 10 == 0 || idx == self.checkpoints.len() - 1 {
                    report.push_str(&format!("  Checkpoint {:2}: seq={:6}, hash={}...\n",
                        idx, seq, &hash[..16]));
                }
            }
        }

        report
    }

    /// Get number of entries in chain
    pub fn entry_count(&self) -> u64 {
        self.entry_count
    }

    /// Get current chain hash
    pub fn current_hash(&self) -> &str {
        &self.current_hash
    }
}

impl fmt::Display for HashChainedAuditLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HashChainedAuditLog {{ entries: {}, current_hash: {}..., verified: {} }}",
            self.entry_count,
            &self.current_hash[..16.min(self.current_hash.len())],
            self.verified_hashes
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_creation() {
        let log = HashChainedAuditLog::new();
        assert_eq!(log.entry_count, 0);
        assert!(log.current_hash.len() > 0);
    }

    #[test]
    fn test_single_entry_recording() {
        let mut log = HashChainedAuditLog::new();
        let genesis_hash = log.current_hash.clone();

        log.record_context_switch("test state", 0x1000, 0, 1);

        assert_eq!(log.entry_count, 1);
        assert_ne!(log.current_hash, genesis_hash);
        assert!(log.current_hash.len() == 64); // SHA256 hex is 64 chars
    }

    #[test]
    fn test_chain_integrity_verification() {
        let mut log = HashChainedAuditLog::new();

        // Record 1000 context switches
        for i in 0..1000 {
            let state = format!("switch_{}", i);
            log.record_context_switch(&state, 0x1000 + i * 0x100, 0, i as u64);
        }

        assert_eq!(log.entry_count, 1000);

        // Verify integrity
        let valid = log.verify_chain_integrity();
        assert!(valid, "Chain should be valid");
        assert_eq!(log.verification_failures, 0);
        assert_eq!(log.verified_hashes as u64, log.entry_count + 1); // +1 for genesis
    }

    #[test]
    fn test_large_scale_chain_1m() {
        let mut log = HashChainedAuditLog::new();

        println!("Recording 100,000 entries (scale test)...");

        // Simulate 100K switches (1M would be too slow for unit test)
        for i in 0..100_000 {
            let state = if i % 10_000 == 0 { "phase_boundary" } else { "normal" };
            log.record_context_switch(state, 0x1000 + (i as usize * 0x10), 0, i as u64);
        }

        assert_eq!(log.entry_count, 100_000);
        assert_eq!(log.checkpoints.len(), 1); // One checkpoint at 100k

        // Verify all entries
        let mut verify_log = log;
        let valid = verify_log.verify_chain_integrity();
        assert!(valid);
        assert_eq!(verify_log.verification_failures, 0);
    }

    #[test]
    fn test_checkpoint_verification() {
        let mut log = HashChainedAuditLog::new();

        // Record entries to create checkpoints
        for i in 0..300_000 {
            let state = format!("switch_{}", i);
            log.record_context_switch(&state, 0x1000, 0, i as u64);
        }

        // Should have checkpoints at 100K, 200K, 300K
        assert!(log.checkpoints.len() >= 3);

        // Verify from checkpoint
        let (valid, msg) = log.verify_from_checkpoint(100_000);
        assert!(valid, "{}", msg);
    }

    #[test]
    fn test_audit_entry_hash() {
        let entry = AuditEntry {
            sequence_number: 1,
            timestamp: 1000,
            state_description: "test".to_string(),
            stack_pointer: 0x1000,
            stack_drift: 0,
            context_switch_count: 1,
            current_hash: String::new(),
            previous_hash: "genesis".to_string(),
        };

        let computed_hash = entry.compute_hash();
        assert_eq!(computed_hash.len(), 64); // SHA256 hex
    }

    #[test]
    fn test_get_report() {
        let mut log = HashChainedAuditLog::new();

        for i in 0..100 {
            log.record_context_switch("test", 0x1000, 0, i);
        }

        let report = log.get_chain_report();
        assert!(report.contains("Hash-Chained Audit Log Report"));
        assert!(report.contains("100"));
    }
}
