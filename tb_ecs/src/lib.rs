#![feature(once_cell)]

pub use component::*;
pub use entity::Entities;
pub use entity::Entity;
pub use join::*;
pub use scheduler::Scheduler;
pub use system::data::SystemData;
pub use system::data::Write;
pub use system::data::RAW;
pub use system::data::RBW;
pub use system::registry::SystemInfo;
pub use system::registry::SystemRegistry;
pub use system::System;
pub use tb_ecs_macro::*;
pub use world::Resource;
pub use world::ResourceId;
pub use world::World;

mod component;
mod entity;
mod join;
mod scheduler;
mod sparse_set;
mod system;
mod world;
