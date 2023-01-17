pub mod docker;
pub mod lxc;

pub use docker::Docker;
pub use lxc::LXC;
pub use prelude::Backend;

pub mod prelude {
    pub use anyhow::{bail, Result};
    pub use async_trait::async_trait;

    #[async_trait]
    pub trait Backend<'a> {
        type CreateResult;
        async fn new(name: &'a str) -> Result<Self>
        where
            Self: Sized;
        async fn create(
            &self,
            image: Option<&'a str>,
            release: Option<&'a str>,
        ) -> Self::CreateResult;
    }
}

pub mod lxc_prelude {
    pub use crate::{backends::prelude::*, lxc};
    pub use std::ptr::{null, null_mut};
    pub fn str_to_cstr(string: &str) -> Result<*const i8> {
        Ok(std::ffi::CString::new(string)?.as_ptr())
    }
}
