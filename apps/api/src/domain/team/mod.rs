// Team domain module
// Contains team aggregate root, value objects, and domain events

#![allow(clippy::module_inception)]

pub mod events;
pub mod team;
pub mod value_objects;

// Re-export main types for convenience
pub use team::Team;
pub use value_objects::TeamStatus;
