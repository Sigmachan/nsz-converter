//! Core conversion engine - wraps nsz CLI subprocess for NSZ/NSP operations

pub mod conversion;
pub mod nsz_wrapper;
pub mod verification;
pub mod runtime;

pub use conversion::{ConversionEngine, ConversionJob, ConversionStatus};
pub use nsz_wrapper::NszWrapper;
pub use verification::VerificationResult;
pub use runtime::{RuntimeManager, Platform};
