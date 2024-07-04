use std::collections::HashMap;

use colette_core::scraper::{Downloader, Extractor, Postprocessor};

pub struct PluginRegistry<T, U> {
    pub downloaders: HashMap<&'static str, Box<dyn Downloader + Send + Sync>>,
    pub extractors: HashMap<&'static str, Box<dyn Extractor<T> + Send + Sync>>,
    pub postprocessors: HashMap<&'static str, Box<dyn Postprocessor<T, U> + Send + Sync>>,
}
