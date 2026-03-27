mod google;
mod local;

pub use google::GoogleProvider;
pub use local::LocalProvider;

use crate::search::SearchOutput;

pub trait Provider {
    fn search(&self, query: &str, lang: (&str, &str)) -> SearchOutput;
    fn name(&self) -> &str;
}
