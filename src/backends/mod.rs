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
        async fn new_async(_: &'a str) -> Result<Self>
        where
            Self: Sized,
        {
            unimplemented!()
        }
        async fn create_async(&self, _: &'a str, _: &'a str) -> Self::CreateResult {
            unimplemented!()
        }

        fn new(_: &'a str) -> Result<Self>
        where
            Self: Sized,
        {
            unimplemented!()
        }
        fn create(&self, _: &'a str, _: &'a str) -> Self::CreateResult {
            unimplemented!()
        }
    }
}

pub mod lxc_prelude {
    pub use crate::{backends::prelude::*, lxc};
    pub use std::ptr::{null, null_mut};

    #[macro_export]
    macro_rules! str_to_cstr {
        ($s:expr) => {
            std::ffi::CString::new($s).unwrap().as_ptr()
        };
    }
}
