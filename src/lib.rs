/**
 * 🐀 Test Mouse: FreeLang OS Kernel Library
 *
 * Modules:
 *   - stack_integrity: Stack Pointer protection and return address validation
 *   - Additional modules for Phase 2-6
 */

pub mod stack_integrity;

pub use stack_integrity::{
    StackIntegrityMonitor,
    StackIntegrityMetrics,
    StackIntegrityValidation,
    ShadowStack,
    StackFrame,
};
