use camino::Utf8PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Cannot get parent path from {0:?}")]
    BadPathNoParent(Utf8PathBuf),
}
