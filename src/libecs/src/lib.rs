extern crate alloc;

#[cfg(target_os = "rust_os")]
mod compat {
    pub use os::Result;
    pub use syscall::ErrNum;
}

#[cfg(not(target_os = "rust_os"))]
mod compat {
    use core::result;

    #[derive(Copy, Clone, Debug)]
    pub enum ErrNum {
        InvalidArgument,
    }

    pub type Result<T> = result::Result<T, ErrNum>;
}

mod archetype;
mod component;
mod entity;
mod system;
mod type_map;

pub use component::ComponentStorage;
pub use entity::Entity;
pub use system::System;
