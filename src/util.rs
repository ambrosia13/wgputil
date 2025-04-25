use std::path::Path;

pub(crate) fn name_from_path<P>(path: P) -> Option<String>
where
    P: AsRef<Path>,
{
    // ew
    Some(path.as_ref().file_name()?.to_str()?.to_owned())
}
