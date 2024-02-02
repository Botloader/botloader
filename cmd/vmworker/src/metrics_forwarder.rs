use std::{collections::HashMap, sync::Arc};

use core::slice::Iter;
use metrics::{CounterFn, GaugeFn, Label, Recorder};
use scheduler_worker_rpc::{CounterEvent, GaugeEvent, MetricEvent, WorkerMessage};
use tokio::sync::mpsc;

pub struct MetricsForwarder {
    pub tx: mpsc::UnboundedSender<WorkerMessage>,
}

impl Recorder for MetricsForwarder {
    fn describe_counter(
        &self,
        _key: metrics::KeyName,
        _unit: Option<metrics::Unit>,
        _description: metrics::SharedString,
    ) {
    }

    fn describe_gauge(
        &self,
        _key: metrics::KeyName,
        _unit: Option<metrics::Unit>,
        _description: metrics::SharedString,
    ) {
    }

    fn describe_histogram(
        &self,
        _key: metrics::KeyName,
        _unit: Option<metrics::Unit>,
        _description: metrics::SharedString,
    ) {
    }

    fn register_counter(
        &self,
        key: &metrics::Key,
        _metadata: &metrics::Metadata<'_>,
    ) -> metrics::Counter {
        metrics::Counter::from_arc(Arc::new(Metric::new(self.tx.clone(), key)))
    }

    fn register_gauge(
        &self,
        key: &metrics::Key,
        _metadata: &metrics::Metadata<'_>,
    ) -> metrics::Gauge {
        metrics::Gauge::from_arc(Arc::new(Metric::new(self.tx.clone(), key)))
    }

    fn register_histogram(
        &self,
        _key: &metrics::Key,
        _metadata: &metrics::Metadata<'_>,
    ) -> metrics::Histogram {
        metrics::Histogram::noop()
    }
    // fn describe_counter(
    //     &self,
    //     _key: metrics::KeyName,
    //     _unit: Option<metrics::Unit>,
    //     _description: &'static str,
    // ) {
    // }

    // fn describe_gauge(
    //     &self,
    //     _key: metrics::KeyName,
    //     _unit: Option<metrics::Unit>,
    //     _description: &'static str,
    // ) {
    // }

    // fn describe_histogram(
    //     &self,
    //     _key: metrics::KeyName,
    //     _unit: Option<metrics::Unit>,
    //     _description: &'static str,
    // ) {
    // }

    // fn register_counter(&self, key: &metrics::Key) -> metrics::Counter {
    //     metrics::Counter::from_arc(Arc::new(Metric::new(self.tx.clone(), key)))
    // }

    // fn register_gauge(&self, key: &metrics::Key) -> metrics::Gauge {
    //     metrics::Gauge::from_arc(Arc::new(Metric::new(self.tx.clone(), key)))
    // }

    // fn register_histogram(&self, _key: &metrics::Key) -> metrics::Histogram {
    //     metrics::Histogram::noop()
    // }
}

struct Metric {
    tx: mpsc::UnboundedSender<WorkerMessage>,
    key: metrics::Key,
}

impl Metric {
    fn new(tx: mpsc::UnboundedSender<WorkerMessage>, key: &metrics::Key) -> Self {
        Self {
            tx,
            key: key.clone(),
        }
    }

    fn send_metric_event(&self, evt: MetricEvent) {
        let hash_map = key_labels_to_hash_map(self.key.labels());

        let _ = self.tx.send(WorkerMessage::Metric(
            self.key.name().to_owned(),
            evt,
            hash_map,
        ));
    }
}

impl GaugeFn for Metric {
    fn increment(&self, value: f64) {
        self.send_metric_event(MetricEvent::Gauge(GaugeEvent::Incr(value)));
    }

    fn decrement(&self, value: f64) {
        self.send_metric_event(MetricEvent::Gauge(GaugeEvent::Incr(-value)));
    }

    fn set(&self, value: f64) {
        self.send_metric_event(MetricEvent::Gauge(GaugeEvent::Set(value)));
    }
}

impl CounterFn for Metric {
    fn increment(&self, value: u64) {
        self.send_metric_event(MetricEvent::Counter(CounterEvent::Incr(value)));
    }

    fn absolute(&self, value: u64) {
        self.send_metric_event(MetricEvent::Counter(CounterEvent::Absolute(value)));
    }
}

fn key_labels_to_hash_map(labels: Iter<Label>) -> HashMap<String, String> {
    labels
        .map(|v| (v.key().to_owned(), v.value().to_owned()))
        .collect()
}
