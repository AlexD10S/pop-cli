pub mod errors;
pub mod git;
pub mod helpers;
pub mod templates;

pub use errors::Error;
pub use git::{Git, GitHub, Release};
pub use helpers::replace_in_file;
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
