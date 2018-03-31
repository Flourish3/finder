use gio::{resources_register, Error, Resource};
use glib::Bytes;

pub fn init() -> Result<(), Error> {
    let res_bytes = include_bytes!("../res/resources.gresource");

    let gbytes = Bytes::from(res_bytes.as_ref());
    let resource = Resource::new_from_data(&gbytes)?;

    resources_register(&resource);

    Ok(())
}