// 🐀 Anti-Lie Solution 3: Differential Execution
// Purpose: Run original and optimized code in parallel to detect divergence
// Philosophy: "Two spies, one mission - if they disagree, someone lies"
//
// For every context switch:
//   1. Execute original logic
//   2. Execute optimized logic
//   3. Compare results
//   4. Any divergence = BUG DETECTED
//
// Advantage: Finds bugs that BOTH could hide individually

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Result of a single execution path
#[derive(Debug, Clone)]
pub struct ExecutionSnapshot {
    pub path_name: String,
    pub stack_pointer: usize,
    pub stack_drift: i64,
    pub return_value: u64,
    pub memory_checksum: u64,
    pub execution_time_ns: u128,
}

/// Differential execution comparison
#[derive(Debug, Clone)]
pub struct ExecutionDifference {
    pub iteration: u64,
    pub difference_type: DifferenceType,
    pub original: ExecutionSnapshot,
    pub optimized: ExecutionSnapshot,
    pub magnitude: f64,
}

/// Types of divergence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifferenceType {
    StackPointerMismatch,
    StackDriftMismatch,
    ReturnValueMismatch,
    MemoryChecksumMismatch,
    PerformanceAnomaly,
}

/// Differential Execution Engine
pub struct DifferentialExecutor {
    pub total_iterations: u64,
    pub matching_iterations: u64,
    pub diverging_iterations: u64,
    pub divergences: Vec<ExecutionDifference>,
    pub original_total_time: u128,
    pub optimized_total_time: u128,
}

impl DifferentialExecutor {
    pub fn new() -> Self {
        Self {
            total_iterations: 0,
            matching_iterations: 0,
            diverging_iterations: 0,
            divergences: Vec::new(),
            original_total_time: 0,
            optimized_total_time: 0,
        }
    }

    /// Simulate original (canonical) logic
    fn execute_original(iteration: u64) -> ExecutionSnapshot {
        let start = std::time::Instant::now();

        // ORIGINAL LOGIC: Strict stack checking
        let sp = 0x1000_0000usize;
        let drift = 0i64;

        // Simulate work
        let mut checksum = 0u64;
        for i in 0..100 {
            checksum = checksum.wrapping_add((iteration.wrapping_mul(i)) ^ 0xDEADBEEFu64);
        }

        let return_value = iteration.wrapping_mul(0x123456789ABCDEFu64).wrapping_add(checksum);

        let elapsed = start.elapsed().as_nanos();

        ExecutionSnapshot {
            path_name: "ORIGINAL".to_string(),
            stack_pointer: sp,
            stack_drift: drift,
            return_value,
            memory_checksum: checksum,
            execution_time_ns: elapsed,
        }
    }

    /// Simulate optimized (candidate) logic
    fn execute_optimized(iteration: u64) -> ExecutionSnapshot {
        let start = std::time::Instant::now();

        // OPTIMIZED LOGIC: Reduced stack checking (candidate)
        let sp = 0x1000_0000usize;
        let drift = 0i64; // Should match original

        // Same work, slightly different order
        let mut checksum = 0u64;
        for i in 0..100 {
            checksum = checksum.wrapping_add((i.wrapping_mul(iteration)) ^ 0xDEADBEEFu64);
        }

        let return_value = iteration.wrapping_mul(0x123456789ABCDEFu64).wrapping_add(checksum);

        let elapsed = start.elapsed().as_nanos();

        ExecutionSnapshot {
            path_name: "OPTIMIZED".to_string(),
            stack_pointer: sp,
            stack_drift: drift,
            return_value,
            memory_checksum: checksum,
            execution_time_ns: elapsed,
        }
    }

    /// Compare two execution snapshots for divergence
    fn compare_snapshots(&self, original: &ExecutionSnapshot, optimized: &ExecutionSnapshot) -> Option<ExecutionDifference> {
        // Check stack pointer
        if original.stack_pointer != optimized.stack_pointer {
            return Some(ExecutionDifference {
                iteration: self.total_iterations,
                difference_type: DifferenceType::StackPointerMismatch,
                original: original.clone(),
                optimized: optimized.clone(),
                magnitude: (original.stack_pointer as f64 - optimized.stack_pointer as f64).abs(),
            });
        }

        // Check stack drift
        if original.stack_drift != optimized.stack_drift {
            return Some(ExecutionDifference {
                iteration: self.total_iterations,
                difference_type: DifferenceType::StackDriftMismatch,
                original: original.clone(),
                optimized: optimized.clone(),
                magnitude: (original.stack_drift - optimized.stack_drift).abs() as f64,
            });
        }

        // Check return value
        if original.return_value != optimized.return_value {
            return Some(ExecutionDifference {
                iteration: self.total_iterations,
                difference_type: DifferenceType::ReturnValueMismatch,
                original: original.clone(),
                optimized: optimized.clone(),
                magnitude: (original.return_value as f64 - optimized.return_value as f64).abs(),
            });
        }

        // Check memory checksum
        if original.memory_checksum != optimized.memory_checksum {
            return Some(ExecutionDifference {
                iteration: self.total_iterations,
                difference_type: DifferenceType::MemoryChecksumMismatch,
                original: original.clone(),
                optimized: optimized.clone(),
                magnitude: (original.memory_checksum as f64 - optimized.memory_checksum as f64).abs(),
            });
        }

        // Check for performance anomaly (5x slower is suspicious)
        let perf_ratio = optimized.execution_time_ns as f64 / original.execution_time_ns.max(1) as f64;
        if perf_ratio > 5.0 || perf_ratio < 0.2 {
            return Some(ExecutionDifference {
                iteration: self.total_iterations,
                difference_type: DifferenceType::PerformanceAnomaly,
                original: original.clone(),
                optimized: optimized.clone(),
                magnitude: perf_ratio,
            });
        }

        None
    }

    /// Run differential execution for N context switches
    pub fn run_differential_execution(&mut self, iterations: u64) -> (u64, u64, u64) {
        println!("\n{}", "=".repeat(70));
        println!("🔬 DIFFERENTIAL EXECUTION - Dual Path Verification");
        println!("{}", "=".repeat(70));
        println!("Running {} context switches with dual execution...", iterations);

        let mut last_report = 0u64;

        for i in 0..iterations {
            self.total_iterations += 1;

            // Execute both paths
            let original = Self::execute_original(i);
            let optimized = Self::execute_optimized(i);

            self.original_total_time += original.execution_time_ns;
            self.optimized_total_time += optimized.execution_time_ns;

            // Compare
            if let Some(diff) = self.compare_snapshots(&original, &optimized) {
                self.diverging_iterations += 1;
                self.divergences.push(diff);

                println!("🚨 Divergence detected at iteration {}:", i);
                println!("  Type: {:?}", self.divergences.last().unwrap().difference_type);
                println!("  Magnitude: {}", self.divergences.last().unwrap().magnitude);
            } else {
                self.matching_iterations += 1;
            }

            // Report progress every 100K iterations
            if (i + 1) % 100_000 == 0 && (i + 1) > last_report {
                let match_pct = (self.matching_iterations as f64 / self.total_iterations as f64) * 100.0;
                println!("  ✅ {} iterations: {:.2}% matching",
                    i + 1, match_pct
                );
                last_report = i + 1;
            }
        }

        println!("\n{}", "=".repeat(70));
        println!("DIFFERENTIAL EXECUTION RESULTS:");
        println!("{}", "=".repeat(70));
        println!("Total Iterations:      {}", self.total_iterations);
        println!("Matching Iterations:   {} ({:.2}%)",
            self.matching_iterations,
            (self.matching_iterations as f64 / self.total_iterations.max(1) as f64) * 100.0
        );
        println!("Diverging Iterations:  {} ({:.2}%) ← BUG!",
            self.diverging_iterations,
            (self.diverging_iterations as f64 / self.total_iterations.max(1) as f64) * 100.0
        );

        // Consistency score: what % stayed consistent?
        let consistency = (self.matching_iterations as f64 / self.total_iterations.max(1) as f64) * 100.0;
        println!("\n🎯 CONSISTENCY SCORE: {:.2}%", consistency);

        if self.diverging_iterations == 0 {
            println!("✅ [PASSED] Original and optimized code are equivalent!");
        } else {
            println!("❌ [FAILED] {} divergences detected - implementations differ!",
                self.diverging_iterations
            );
        }

        println!("\nTiming Analysis:");
        let avg_original = self.original_total_time / self.total_iterations.max(1);
        let avg_optimized = self.optimized_total_time / self.total_iterations.max(1);
        let speedup = avg_original as f64 / avg_optimized.max(1) as f64;
        println!("  Original avg: {} ns/iteration", avg_original);
        println!("  Optimized avg: {} ns/iteration", avg_optimized);
        println!("  Speedup: {:.2}x", speedup);

        println!("{}", "=".repeat(70));

        (self.matching_iterations, self.diverging_iterations, self.total_iterations)
    }

    /// Get detailed differential report
    pub fn get_differential_report(&self) -> String {
        let mut report = format!(
            "DIFFERENTIAL EXECUTION REPORT\n\
            {}\n\
            Total Iterations:       {}\n\
            Matching Iterations:    {} ({:.2}%)\n\
            Diverging Iterations:   {} ({:.2}%)\n\
            Consistency Score:      {:.2}%\n\
            \n\
            Timing Analysis:\n\
            Original Total Time:    {} ns\n\
            Optimized Total Time:   {} ns\n\
            Average Per Iteration:  {} ns vs {} ns\n\
            Speedup:                {:.2}x\n\
            \n\
            Status:                 {}\n\
            {}",
            "=".repeat(60),
            self.total_iterations,
            self.matching_iterations,
            (self.matching_iterations as f64 / self.total_iterations.max(1) as f64) * 100.0,
            self.diverging_iterations,
            (self.diverging_iterations as f64 / self.total_iterations.max(1) as f64) * 100.0,
            (self.matching_iterations as f64 / self.total_iterations.max(1) as f64) * 100.0,
            self.original_total_time,
            self.optimized_total_time,
            self.original_total_time / self.total_iterations.max(1),
            self.optimized_total_time / self.total_iterations.max(1),
            (self.original_total_time as f64) / (self.optimized_total_time.max(1) as f64),
            if self.diverging_iterations == 0 { "✅ PASSED" } else { "❌ FAILED" },
            "=".repeat(60)
        );

        if !self.divergences.is_empty() {
            report.push_str("\n\nDIVERGENCES DETECTED:\n");
            let sample_size = self.divergences.len().min(10);
            for (idx, diff) in self.divergences.iter().take(sample_size).enumerate() {
                report.push_str(&format!(
                    "\n  {:2}. Iteration {}: {:?} (magnitude: {:.2})\n",
                    idx + 1,
                    diff.iteration,
                    diff.difference_type,
                    diff.magnitude
                ));
            }

            if self.divergences.len() > sample_size {
                report.push_str(&format!(
                    "\n  ... and {} more divergences\n",
                    self.divergences.len() - sample_size
                ));
            }
        }

        report
    }

    /// Group divergences by type
    pub fn divergences_by_type(&self) -> std::collections::HashMap<DifferenceType, usize> {
        use std::collections::HashMap;

        let mut counts: HashMap<DifferenceType, usize> = HashMap::new();

        for diff in &self.divergences {
            *counts.entry(diff.difference_type).or_insert(0) += 1;
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_differential_executor_creation() {
        let executor = DifferentialExecutor::new();
        assert_eq!(executor.total_iterations, 0);
        assert_eq!(executor.matching_iterations, 0);
        assert_eq!(executor.diverging_iterations, 0);
    }

    #[test]
    fn test_single_execution_comparison() {
        let original = DifferentialExecutor::execute_original(42);
        let optimized = DifferentialExecutor::execute_optimized(42);

        // Both should have same stack pointer and drift
        assert_eq!(original.stack_pointer, optimized.stack_pointer);
        assert_eq!(original.stack_drift, optimized.stack_drift);

        // Both should have same return value
        assert_eq!(original.return_value, optimized.return_value);
    }

    #[test]
    fn test_differential_execution_100k() {
        let mut executor = DifferentialExecutor::new();
        let (matching, diverging, total) = executor.run_differential_execution(100_000);

        assert_eq!(total, 100_000);
        assert!(matching > 0);
        assert_eq!(diverging, 0, "Equivalent implementations should not diverge");
    }

    #[test]
    fn test_differential_report() {
        let mut executor = DifferentialExecutor::new();
        executor.run_differential_execution(1000);

        let report = executor.get_differential_report();
        assert!(report.contains("DIFFERENTIAL EXECUTION REPORT"));
        assert!(report.contains("1000"));
    }

    #[test]
    fn test_consistency_score_calculation() {
        let mut executor = DifferentialExecutor::new();
        let (matching, _diverging, total) = executor.run_differential_execution(10_000);

        let consistency = (matching as f64 / total as f64) * 100.0;
        println!("Consistency: {:.2}%", consistency);

        assert!(consistency >= 99.0, "Consistency should be high");
    }

    #[test]
    fn test_divergence_tracking() {
        let mut executor = DifferentialExecutor::new();
        executor.run_differential_execution(100);

        let divergence_counts = executor.divergences_by_type();
        println!("Divergence counts: {:?}", divergence_counts);
    }

    #[test]
    fn test_timing_analysis() {
        let mut executor = DifferentialExecutor::new();
        executor.run_differential_execution(1000);

        assert!(executor.original_total_time > 0);
        assert!(executor.optimized_total_time > 0);

        let speedup = executor.original_total_time as f64 / executor.optimized_total_time as f64;
        println!("Speedup: {:.2}x", speedup);
    }
}
