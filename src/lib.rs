#[cfg(feature = "dependency")]
pub mod dependency_queue;

pub mod israeli_priority;
pub mod my_priority_queue;
pub mod nested_queue;
pub mod no_priority_queue;
pub mod ordinary_priority_queue;

pub use israeli_priority::{Friendly, IsraeliPriority};
pub use my_priority_queue::AbstractPriorityQueue;
pub use nested_queue::BucketQueue;