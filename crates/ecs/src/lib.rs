//! A minimal entity-component system (ECS), written from scratch for learning.
//!
//! An ECS splits a game world into three kinds of things:
//!
//! - **Entities** are plain identifiers ("thing number 42"). They own no data and no behavior.
//! - **Components** are plain data attached to entities: a position, a sprite, a velocity.
//! - **Systems** are functions that run over every entity that has a particular set of
//!   components, for example "move everything that has both a position and a velocity".
//!
//! The crate is intentionally empty for now: it is built in milestone M5 (see `ROADMAP.md`).
//! It exists from day one so that the workspace has its final shape and the dependency
//! `engine` → `ecs` is already in place.

/// The version of this crate, read from its `Cargo.toml` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
