//! Pulsar VPN daemon — бинарь-обёртка вокруг gRPC-сервиса (только Windows).
//!
//! На Windows `pulsar-daemon.exe` поднимает gRPC-сервер на 127.0.0.1:50051
//! и оборачивает [`VpnEngine`] из `pulsar_daemon` в tonic-сервис.
//!
//! На Android этот бинарь не компилируется (`#[cfg(target_os = "windows")]`),
//! и Tauri-приложение использует библиотеку напрямую.

#![cfg(target_os = "windows")]

use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

use pulsar_daemon::vpn_engine::{ConnectRequest, VpnEngine, VpnState};
use pulsar_shared::vpn::vpn_service_server::{VpnService, VpnServiceServer};
use pulsar_shared::vpn::{ConnectionRequest, Empty, Metrics, StatusResponse};

use pulsar_daemon::logger;
use pulsar_daemon::vpn_engine::MetricsEvent;

/// gRPC-обёртка над [`VpnEngine`].
pub struct MyVpnService {
    engine: VpnEngine,
    expected_token: Option<Arc<str>>,
    shutdown_tx: mpsc::Sender<()>,
}

fn check_token<T>(request: &Request<T>, expected: Option<&Arc<str>>) -> Result<(), Status> {
    let Some(token) = expected else { return Ok(()) };
    let got = request.metadata().get("x-pulsar-token").and_then(|v| v.to_str().ok());
    if got != Some(token.as_ref()) {
        return Err(Status::unauthenticated("invalid token"));
    }
    Ok(())
}

fn state_to_proto(s: &VpnState) -> (String, String) {
    match s {
        VpnState::Disconnected => ("Disconnected".into(), "OK".into()),
        VpnState::Connecting => ("Connecting".into(), "OK".into()),
        VpnState::Connected => ("Connected".into(), "OK".into()),
        VpnState::Disconnecting => ("Disconnecting".into(), "OK".into()),
        VpnState::Error(e) => ("Error".into(), e.clone()),
    }
}

#[tonic::async_trait]
impl VpnService for MyVpnService {
    type StreamMetricsStream = ReceiverStream<Result<Metrics, Status>>;

    async fn toggle_connection(
        &self,
        request: Request<ConnectionRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        check_token(&request, self.expected_token.as_ref())?;
        let req = request.into_inner();
        let routing = req.routing_config.unwrap_or_default();
        let engine_req = ConnectRequest { profile_link: req.profile_link, routing };
        let res = if req.connect {
            self.engine.connect(engine_req).await
        } else {
            self.engine.disconnect().await
        };
        let (state, msg) = match res {
            Ok(s) => state_to_proto(&s),
            Err(e) => ("Error".into(), e),
        };
        Ok(Response::new(StatusResponse { state, message: msg }))
    }

    async fn get_status(&self, request: Request<Empty>) -> Result<Response<StatusResponse>, Status> {
        check_token(&request, self.expected_token.as_ref())?;
        let (state, msg) = state_to_proto(&self.engine.state().await);
        Ok(Response::new(StatusResponse { state, message: msg }))
    }

    async fn stream_metrics(&self, request: Request<Empty>) -> Result<Response<Self::StreamMetricsStream>, Status> {
        check_token(&request, self.expected_token.as_ref())?;
        let (tx, rx) = mpsc::channel::<Result<Metrics, Status>>(4);
        let engine = self.engine.clone();
        tokio::spawn(async move {
            // Подписываемся на события метрик.
            let Some(mut metrics_rx) = engine.subscribe_metrics().await else {
                tracing::warn!("stream_metrics: no subscriber available");
                return;
            };
            while let Some(ev) = metrics_rx.recv().await {
                let MetricsEvent { download_speed, upload_speed } = ev;
                let m = Metrics {
                    download_speed,
                    upload_speed,
                };
                if tx.send(Ok(m)).await.is_err() { break; }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn shutdown(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        check_token(&request, self.expected_token.as_ref())?;
        let _ = self.engine.disconnect().await;
        let _ = self.shutdown_tx.send(()).await;
        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init_logger()?;
    let addr = "127.0.0.1:50051".parse()?;
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
    let expected_token = std::env::args().nth(1).map(Arc::from);

    let engine = VpnEngine::new()?;
    let service = MyVpnService { engine, expected_token, shutdown_tx };

    tracing::info!("pulsar-daemon listening on {addr}");
    Server::builder()
        .add_service(VpnServiceServer::new(service))
        .serve_with_shutdown(addr, async move {
            let _ = shutdown_rx.recv().await;
            tracing::info!("pulsar-daemon shutting down");
        })
        .await?;
    Ok(())
}
