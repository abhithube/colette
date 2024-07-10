use std::{collections::HashMap, sync::Arc};

use colette_core::scraper::{Downloader, Extractor, Postprocessor};

pub struct PluginRegistry<T, U> {
    pub downloaders: HashMap<&'static str, Arc<dyn Downloader + Send + Sync>>,
    pub extractors: HashMap<&'static str, Arc<dyn Extractor<T> + Send + Sync>>,
    pub postprocessors: HashMap<&'static str, Arc<dyn Postprocessor<T, U> + Send + Sync>>,
}
