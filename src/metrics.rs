// Branch: metrics-telemetry
// File: src/metrics.rs

use once_cell::sync::Lazy;
use prometheus::{IntCounter, IntGauge, Encoder, TextEncoder, register_int_counter, register_int_gauge};
use std::net::SocketAddr;
use warp::Filter;

pub static TASKS_EXECUTED: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("tasks_executed_total", "Total number of tasks executed").unwrap()
});

pub static ACTIVE_UNITS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!("active_nanobot_units", "Current number of active nanobot units").unwrap()
});

pub async fn serve_metrics(addr: SocketAddr) {
    let metrics_route = warp::path("metrics").map(|| {
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        warp::http::Response::builder()
            .header("Content-Type", encoder.format_type())
            .body(String::from_utf8(buffer).unwrap())
    });

    warp::serve(metrics_route).run(addr).await;
}

