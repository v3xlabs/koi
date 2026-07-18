use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    task::{Context, Poll},
    time::Instant,
};

use alloy::{
    rpc::json_rpc::{RequestPacket, ResponsePacket},
    transports::{TransportError, TransportFut},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tower::{Layer, Service};
use ts_rs::TS;

#[derive(Clone, Debug)]
pub struct RpcMetrics {
    inner: Arc<RpcMetricsInner>,
}

#[derive(Debug)]
struct RpcMetricsInner {
    recent_sample_limit: usize,
    in_flight: AtomicU64,
    queued: AtomicU64,
    max_in_flight: AtomicU64,
    total_requests: AtomicU64,
    total_errors: AtomicU64,
    total_rate_limited: AtomicU64,
    total_duration_ms: AtomicU64,
    connection_attempts: AtomicU64,
    connection_successes: AtomicU64,
    connection_failures: AtomicU64,
    last_request_at: AtomicU64,
    last_error_at: AtomicU64,
    last_connected_at: AtomicU64,
    methods: Mutex<HashMap<String, RpcMethodCounter>>,
    recent: Mutex<VecDeque<RpcCallSample>>,
    last_error: Mutex<Option<String>>,
}

#[derive(Clone, Debug, Default)]
struct RpcMethodCounter {
    total: u64,
    errors: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(optional_fields)]
pub struct RpcEndpointStats {
    pub in_flight: u64,
    pub queued: u64,
    pub max_in_flight: u64,
    pub total_requests: u64,
    pub total_errors: u64,
    pub total_rate_limited: u64,
    pub average_duration_ms: u64,
    pub connection_attempts: u64,
    pub connection_successes: u64,
    pub connection_failures: u64,
    pub last_request_at: Option<u64>,
    pub last_error_at: Option<u64>,
    pub last_connected_at: Option<u64>,
    pub last_error: Option<String>,
    pub methods: Vec<RpcMethodStats>,
    pub recent: Vec<RpcCallSample>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
pub struct RpcMethodStats {
    pub method: String,
    pub total: u64,
    pub errors: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(optional_fields)]
pub struct RpcCallSample {
    pub timestamp: u64,
    pub methods: Vec<String>,
    pub request_count: u64,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RpcMetricsLayer {
    metrics: RpcMetrics,
    semaphore: Arc<Semaphore>,
}

#[derive(Clone, Debug)]
pub struct RpcMetricsService<S> {
    inner: S,
    metrics: RpcMetrics,
    semaphore: Arc<Semaphore>,
}

impl RpcMetrics {
    pub fn new(recent_sample_limit: usize) -> Self {
        Self {
            inner: Arc::new(RpcMetricsInner {
                recent_sample_limit: recent_sample_limit.max(1),
                in_flight: AtomicU64::new(0),
                queued: AtomicU64::new(0),
                max_in_flight: AtomicU64::new(0),
                total_requests: AtomicU64::new(0),
                total_errors: AtomicU64::new(0),
                total_rate_limited: AtomicU64::new(0),
                total_duration_ms: AtomicU64::new(0),
                connection_attempts: AtomicU64::new(0),
                connection_successes: AtomicU64::new(0),
                connection_failures: AtomicU64::new(0),
                last_request_at: AtomicU64::new(0),
                last_error_at: AtomicU64::new(0),
                last_connected_at: AtomicU64::new(0),
                methods: Mutex::new(HashMap::new()),
                recent: Mutex::new(VecDeque::with_capacity(recent_sample_limit.max(1))),
                last_error: Mutex::new(None),
            }),
        }
    }

    pub fn note_connection_attempt(&self) {
        self.inner
            .connection_attempts
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn note_connection_success(&self) {
        self.inner
            .connection_successes
            .fetch_add(1, Ordering::Relaxed);
        self.inner
            .last_connected_at
            .store(now_millis(), Ordering::Relaxed);
    }

    pub fn note_connection_failure(&self, error: String) {
        self.inner
            .connection_failures
            .fetch_add(1, Ordering::Relaxed);
        self.record_error(error);
    }

    pub fn snapshot(&self) -> RpcEndpointStats {
        let total_requests = self.inner.total_requests.load(Ordering::Relaxed);
        let methods = {
            let methods = self
                .inner
                .methods
                .lock()
                .expect("rpc methods mutex poisoned");
            let mut methods = methods
                .iter()
                .map(|(method, stats)| RpcMethodStats {
                    method: method.clone(),
                    total: stats.total,
                    errors: stats.errors,
                })
                .collect::<Vec<_>>();
            methods.sort_by(|a, b| b.total.cmp(&a.total).then_with(|| a.method.cmp(&b.method)));
            methods.truncate(8);
            methods
        };

        RpcEndpointStats {
            in_flight: self.inner.in_flight.load(Ordering::Relaxed),
            queued: self.inner.queued.load(Ordering::Relaxed),
            max_in_flight: self.inner.max_in_flight.load(Ordering::Relaxed),
            total_requests,
            total_errors: self.inner.total_errors.load(Ordering::Relaxed),
            total_rate_limited: self.inner.total_rate_limited.load(Ordering::Relaxed),
            average_duration_ms: if total_requests == 0 {
                0
            } else {
                self.inner.total_duration_ms.load(Ordering::Relaxed) / total_requests
            },
            connection_attempts: self.inner.connection_attempts.load(Ordering::Relaxed),
            connection_successes: self.inner.connection_successes.load(Ordering::Relaxed),
            connection_failures: self.inner.connection_failures.load(Ordering::Relaxed),
            last_request_at: non_zero_millis(self.inner.last_request_at.load(Ordering::Relaxed)),
            last_error_at: non_zero_millis(self.inner.last_error_at.load(Ordering::Relaxed)),
            last_connected_at: non_zero_millis(
                self.inner.last_connected_at.load(Ordering::Relaxed),
            ),
            last_error: self
                .inner
                .last_error
                .lock()
                .expect("rpc last error mutex poisoned")
                .clone(),
            methods,
            recent: self
                .inner
                .recent
                .lock()
                .expect("rpc recent mutex poisoned")
                .iter()
                .cloned()
                .collect(),
        }
    }

    fn record_start(&self, methods: &[String], request_count: u64) {
        let in_flight = self.inner.in_flight.fetch_add(1, Ordering::Relaxed) + 1;
        self.inner
            .max_in_flight
            .fetch_max(in_flight, Ordering::Relaxed);
        self.inner
            .total_requests
            .fetch_add(request_count, Ordering::Relaxed);
        self.inner
            .last_request_at
            .store(now_millis(), Ordering::Relaxed);

        let mut counters = self
            .inner
            .methods
            .lock()
            .expect("rpc methods mutex poisoned");
        for method in methods {
            counters.entry(method.clone()).or_default().total += 1;
        }
    }

    fn record_finish(
        &self,
        methods: Vec<String>,
        request_count: u64,
        duration_ms: u64,
        result: &Result<ResponsePacket, TransportError>,
    ) {
        self.inner.in_flight.fetch_sub(1, Ordering::Relaxed);
        self.inner
            .total_duration_ms
            .fetch_add(duration_ms.saturating_mul(request_count), Ordering::Relaxed);

        let error = match result {
            Ok(response) => response.as_error().map(ToString::to_string),
            Err(error) => Some(error.to_string()),
        };

        if let Some(error) = &error {
            let count = match result {
                Ok(response) => response.iter_errors().count().max(1) as u64,
                Err(_) => 1,
            };
            self.inner.total_errors.fetch_add(count, Ordering::Relaxed);
            if is_rate_limit_error(error) {
                self.inner
                    .total_rate_limited
                    .fetch_add(count, Ordering::Relaxed);
            }

            let mut counters = self
                .inner
                .methods
                .lock()
                .expect("rpc methods mutex poisoned");
            for method in &methods {
                counters.entry(method.clone()).or_default().errors += 1;
            }
            self.record_error(error.to_string());
        }

        self.push_sample(RpcCallSample {
            timestamp: now_millis(),
            methods,
            request_count,
            duration_ms,
            success: error.is_none(),
            error,
        });
    }

    fn record_error(&self, error: String) {
        self.inner
            .last_error_at
            .store(now_millis(), Ordering::Relaxed);
        *self
            .inner
            .last_error
            .lock()
            .expect("rpc last error mutex poisoned") = Some(error);
    }

    fn push_sample(&self, sample: RpcCallSample) {
        let mut recent = self.inner.recent.lock().expect("rpc recent mutex poisoned");
        if recent.len() == self.inner.recent_sample_limit {
            recent.pop_front();
        }
        recent.push_back(sample);
    }
}

impl RpcMetricsLayer {
    pub fn new(metrics: RpcMetrics, max_in_flight: usize) -> Self {
        Self {
            metrics,
            semaphore: Arc::new(Semaphore::new(max_in_flight.max(1))),
        }
    }
}

impl<S> Layer<S> for RpcMetricsLayer {
    type Service = RpcMetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RpcMetricsService {
            inner,
            metrics: self.metrics.clone(),
            semaphore: self.semaphore.clone(),
        }
    }
}

impl<S> Service<RequestPacket> for RpcMetricsService<S>
where
    S: Service<RequestPacket, Response = ResponsePacket, Error = TransportError>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: RequestPacket) -> Self::Future {
        let metrics = self.metrics.clone();
        let semaphore = self.semaphore.clone();
        let mut inner = self.inner.clone();
        let methods = request
            .method_names()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let request_count = request.len() as u64;

        Box::pin(async move {
            metrics.inner.queued.fetch_add(1, Ordering::Relaxed);
            let permit = semaphore
                .acquire_owned()
                .await
                .expect("rpc semaphore closed unexpectedly");
            metrics.inner.queued.fetch_sub(1, Ordering::Relaxed);

            metrics.record_start(&methods, request_count);
            let started = Instant::now();
            let result = inner.call(request).await;
            let duration_ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
            drop(permit);
            metrics.record_finish(methods, request_count, duration_ms, &result);
            result
        })
    }
}

fn now_millis() -> u64 {
    Utc::now().timestamp_millis() as u64
}

fn non_zero_millis(value: u64) -> Option<u64> {
    if value == 0 { None } else { Some(value) }
}

fn is_rate_limit_error(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("rate")
        || error.contains("limit")
        || error.contains("too many")
        || error.contains("429")
        || error.contains("exceeded")
}
