//! Raw-disk snapshot subsystem (remote `dd`).

mod builder; // config --> validated config
mod meta;
mod pipeline; // streaming copy + hash + json
mod probe; // lsblk device discovery // serialisable metadata

pub use builder::{DdBuilder, ResumeMode};

pub use pipeline::run_once;
