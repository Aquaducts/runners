use crate::lxc::lxc_container;

use super::lxc_prelude::*;
use anyhow::Result;

pub struct InnerLxcContainer(*mut lxc_container);

unsafe impl Sync for InnerLxcContainer {}

pub struct LXC<'a> {
    pub name: &'a str,
    pub _inner: InnerLxcContainer,
}

#[async_trait]
impl<'a> Backend<'a> for LXC<'a> {
    type CreateResult = Result<()>;

    async fn new(name: &'a str) -> Result<Self> {
        unsafe {
            let c_str_name = str_to_cstr(name)?;
            let container = lxc::lxc_container_new(c_str_name, null());
            Ok(Self {
                name,
                _inner: InnerLxcContainer(container),
            })
        }
    }

    async fn create(&self, image: Option<&'a str>, release: Option<&'a str>) -> Self::CreateResult {
        unsafe {
            let container = self._inner.0;
            let (Some(image), Some(release)) = (image, release) else {
                bail!("Both image and release are required.");
            };
            let Some(createl) = (*container).createl else {
                bail!("Could not find LXC createl function.");
            };
            createl(
                container,
                str_to_cstr("download")?,
                null(),
                null_mut(),
                lxc::LXC_CREATE_QUIET as i32,
                str_to_cstr("-d")?,
                str_to_cstr(image)?,
                str_to_cstr("-r")?,
                str_to_cstr(release)?,
                str_to_cstr("-a")?,
                str_to_cstr("amd64")?,
                null() as *const i8,
            );
            Ok(())
        }
    }
}
