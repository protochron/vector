use crate::{
    api::schema::metrics::{self, MetricsFilter},
    event::Metric,
};
use async_graphql::Object;
use std::collections::BTreeMap;

pub struct FileSourceMetricFile<'a> {
    name: String,
    metrics: Vec<&'a Metric>,
}

impl<'a> FileSourceMetricFile<'a> {
    /// Returns a new FileSourceMetricFile from a (name, Vec<&Metric>) tuple
    fn from_tuple((name, metrics): (String, Vec<&'a Metric>)) -> Self {
        Self { name, metrics }
    }
}

#[Object]
impl FileSourceMetricFile<'_> {
    /// File name
    async fn name(&self) -> &str {
        &*self.name
    }

    /// Metric indicating events processed for the current file
    async fn processed_events_total(&self) -> Option<metrics::ProcessedEventsTotal> {
        self.metrics.processed_events_total()
    }

    /// Metric indicating bytes processed for the current file
    async fn processed_bytes_total(&self) -> Option<metrics::ProcessedBytesTotal> {
        self.metrics.processed_bytes_total()
    }
}

#[derive(Debug, Clone)]
pub struct FileSourceMetrics(Vec<Metric>);

impl FileSourceMetrics {
    pub fn new(metrics: Vec<Metric>) -> Self {
        Self(metrics)
    }
}

#[Object]
impl FileSourceMetrics {
    /// File metrics
    pub async fn files(&self) -> Vec<FileSourceMetricFile<'_>> {
        self.0
            .iter()
            .filter_map(|m| match m.tag_value("file") {
                Some(file) => Some((file, m)),
                _ => None,
            })
            .fold(BTreeMap::new(), |mut map, (file, m)| {
                map.entry(file).or_insert_with(Vec::new).push(m);
                map
            })
            .into_iter()
            .map(FileSourceMetricFile::from_tuple)
            .collect()
    }

    /// Events processed for the current file source
    pub async fn processed_events_total(&self) -> Option<metrics::ProcessedEventsTotal> {
        self.0.processed_events_total()
    }

    /// Bytes processed for the current file source
    pub async fn processed_bytes_total(&self) -> Option<metrics::ProcessedBytesTotal> {
        self.0.processed_bytes_total()
    }
}
