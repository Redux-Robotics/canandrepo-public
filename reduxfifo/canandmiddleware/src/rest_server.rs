use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
};
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use tokio::sync::watch;
use tower_http::cors::{Any, CorsLayer};

use crate::log::*;
use crate::ota::{OtaAddress, OtaTask};
use crate::{
    backend::{self, FIFOCoreError},
    bus::{self, BusState, device::DeviceType},
};
use fifocore::{FIFOCore, ReduxFIFOSessionConfig, error::Error};

// -----------------------

const fn banner() -> &'static str {
    const_format::formatcp!(
        r#"<html lang="en">
<head>
<meta charset="UTF-8">
</head>
<body>
<pre>
    ____           __              ____        __          __  _          
   / __ \___  ____/ /_  ___  __   / __ \____  / /_  ____  / /_(_)_________
  / /_/ / _ \/ __  / / / / |/_/  / /_/ / __ \/ __ \/ __ \/ __/ / ___/ ___/
 / _, _/  __/ /_/ / /_/ />  <   / _, _/ /_/ / /_/ / /_/ / /_/ / /__(__  ) 
/_/ |_|\___/\__,_/\__,_/_/|_|  /_/ |_|\____/_.___/\____/\__/_/\___/____/  
</pre>

<p>ReduxCANLink is running with canandmiddlware.</p>
<p>ReduxFIFO version: {}</p>
</body>
</html>"#,
        env!("CARGO_PKG_VERSION")
    )
}

// Application state
#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) fifocore: FIFOCore,
    pub(crate) ota_clients: Arc<Mutex<FxHashMap<OtaAddress, OtaTask>>>,
    pub(crate) bus_sessions: Arc<Mutex<FxHashMap<u16, BusState>>>,
}

// These are in order of their `.route` definitions

/// `/version`
async fn version_handler() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
/// `/`
async fn banner_handler() -> Html<&'static str> {
    Html(banner())
}

/// `/configurator`
async fn configurator_handler() -> Html<&'static str> {
    Html(include_str!("html/configurator.html"))
}

/// `/ws/{bus}`
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(bus_id): Path<u16>,
) -> axum::response::Response {
    let fifocore = state.fifocore;
    ws.on_upgrade(move |socket| crate::websocket::handle_socket(socket, fifocore, bus_id))
}

/// `/buses`
async fn list_bus_handler(State(state): State<AppState>) -> Json<backend::ListBuses> {
    Json(backend::handle_list_bus(&state.fifocore))
}

/// `/buses/open?params=...` where `params` is the bus open params
async fn open_bus_handler(
    State(state): State<AppState>,
    Query(params): Query<FxHashMap<String, String>>,
) -> axum::response::Response {
    let Some(bus_name) = params.get("params") else {
        let mut response = Json(()).into_response();
        *response.status_mut() = StatusCode::BAD_REQUEST;
        return response;
    };
    backend::handle_open_bus(&state.fifocore, bus_name)
}

fn sessions_open_bus_inner<'a>(
    mut bus_sessions: parking_lot::MutexGuard<'a, FxHashMap<u16, BusState>>,
    state: &AppState,
    bus_id: u16,
) -> Result<(), Json<FIFOCoreError>> {
    let config = ReduxFIFOSessionConfig::new(0x0e0000, 0xff0000);
    let session = state
        .fifocore
        .open_managed_session(bus_id, 256, config)
        .map_err(|e| Json::<FIFOCoreError>(e.into()))?;
    let (start_send, start_gate) = tokio::sync::oneshot::channel();

    let task = tokio::task::spawn(bus::bus_session(
        start_gate,
        session,
        state.bus_sessions.clone(),
    ));
    bus_sessions.insert(bus_id, BusState::new(task, state.fifocore.clone(), bus_id));
    drop(bus_sessions);
    let _ = start_send.send(());
    Ok(())
}

/// `sessions/open/{bus}`
async fn session_open_bus(
    State(state): State<AppState>,
    Path(bus_id): Path<u16>,
) -> Result<Json<()>, Json<FIFOCoreError>> {
    if !state.fifocore.buses().contains(&bus_id) {
        return Err(Json(backend::FIFOCoreError::from(Error::InvalidBus)));
    };
    let bus_sessions = state.bus_sessions.lock();
    if !bus_sessions.contains_key(&bus_id) {
        sessions_open_bus_inner(bus_sessions, &state, bus_id)?;
    }
    Ok(Json(()))
}

/// `sessions/close/{bus}`
async fn session_close_bus(State(state): State<AppState>, Path(bus_id): Path<u16>) -> Json<()> {
    let mut bus_sessions = state.bus_sessions.lock();
    drop(bus_sessions.remove(&bus_id));
    Json(())
}

/// `sessions/{bus}/enumerate`
async fn session_enumerate_bus(
    State(state): State<AppState>,
    Path(bus_id): Path<u16>,
) -> Result<Json<()>, Json<FIFOCoreError>> {
    let mut bus_sessions = state.bus_sessions.lock();
    let Some(state) = bus_sessions.get_mut(&bus_id) else {
        return Err(Json(fifocore::error::Error::InvalidBus.into()));
    };
    state.enumerate().map_err(|e| Json(e.into()))?;
    Ok(Json(()))
}

/// `sessions/{bus}/devices/list`
async fn session_list_devices(
    State(state): State<AppState>,
    Path(bus_id): Path<u16>,
) -> Result<Json<FxHashMap<String, DeviceType>>, Json<FIFOCoreError>> {
    let bus_sessions = state.bus_sessions.lock();
    if let Some(state) = bus_sessions.get(&bus_id) {
        Ok(Json(state.known_devices()))
    } else {
        sessions_open_bus_inner(bus_sessions, &state, bus_id)?;
        Ok(Json(FxHashMap::default()))
    }
}

/// `sessions/{bus}/devices/clear`
async fn session_clear_devices(
    State(state): State<AppState>,
    Path(bus_id): Path<u16>,
) -> Result<Json<()>, StatusCode> {
    let mut bus_sessions = state.bus_sessions.lock();
    let Some(state) = bus_sessions.get_mut(&bus_id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    state.clear_known_devices();
    Ok(Json(()))
}

/// `sessions/{bus}/devices/arbitrate?serial=`
async fn session_arb_device(
    State(state): State<AppState>,
    Path((bus_id, device_id_hex)): Path<(u16, String)>,
    Query(params): Query<FxHashMap<String, String>>,
) -> Result<Json<()>, StatusCode> {
    let device_id = session_hex(&device_id_hex)?;
    let serial_numer = pull_key(&params, "serial", |v| {
        serial_numer::SerialNumer::from_readable_str(v, true)
    })?;

    let mut bus_sessions = state.bus_sessions.lock();
    let state = bus_state(&mut bus_sessions, bus_id)?;

    state.arbitrate(device_id, serial_numer).map_err(|e| {
        log_error!("Couldn't arbitrate ids on {device_id_hex}: {e}!");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(()))
}

/// `sessions/{bus}/devices/{device}/blink?r=1`
async fn session_blink_device(
    State(state): State<AppState>,
    Path((bus_id, device_id_hex)): Path<(u16, String)>,
    Query(params): Query<FxHashMap<String, u8>>,
) -> Result<Json<()>, StatusCode> {
    let device_id = session_hex(&device_id_hex)?;
    let value = pull_key(&params, "r", |v| Some(*v))?;

    let mut bus_sessions = state.bus_sessions.lock();
    let state = bus_state(&mut bus_sessions, bus_id)?;

    state.blink(device_id, value).map_err(|e| {
        log_error!("Couldn't blink LED: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(()))
}

/// `sessions/{bus}/devices/{device}/set_id?id=1`
async fn session_set_id_device(
    State(state): State<AppState>,
    Path((bus_id, device_id_hex)): Path<(u16, String)>,
    Query(params): Query<FxHashMap<String, u8>>,
) -> Result<Json<()>, StatusCode> {
    let device_id = session_hex(&device_id_hex)?;
    let new_id = pull_key(&params, "id", |v| Some(*v))?;

    let mut bus_sessions = state.bus_sessions.lock();
    let state = bus_state(&mut bus_sessions, bus_id)?;
    state.set_id(device_id, new_id).map_err(|e| {
        log_error!("Couldn't set device ID on {device_id_hex}: {e}!");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(()))
}

async fn session_fetch_setting(
    State(state): State<AppState>,
    Path((bus_id, device_id_hex)): Path<(u16, String)>,
    Query(params): Query<FxHashMap<String, String>>,
) -> Result<Json<Option<crate::bus::FetchSetting>>, StatusCode> {
    let device_id = session_hex(&device_id_hex)?;
    let index = pull_key(&params, "index", |v| v.parse::<u8>().ok())?;

    {
        let mut bus_sessions = state.bus_sessions.lock();
        let state = bus_state(&mut bus_sessions, bus_id)?;
        state.send_fetch_setting(device_id, index).map_err(|e| {
            log_error!("Couldn't set device ID on {device_id_hex}: {e}!");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    tokio::time::sleep(Duration::from_millis(
        params
            .get("wait")
            .and_then(|w| w.parse::<u64>().ok())
            .unwrap_or(50),
    ))
    .await;

    {
        let mut bus_sessions = state.bus_sessions.lock();
        Ok(Json(bus_sessions.get_mut(&bus_id).and_then(|bus_state| {
            bus_state.setting_cache(device_id, index)
        })))
    }
}

async fn session_set_name(
    State(state): State<AppState>,
    Path((bus_id, device_id_hex)): Path<(u16, String)>,
    Query(params): Query<FxHashMap<String, String>>,
) -> Result<Json<()>, StatusCode> {
    let device_id = session_hex(&device_id_hex)?;
    let name: String = pull_key(&params, "name", |v| Some(v.clone()))?;
    {
        let mut bus_sessions = state.bus_sessions.lock();
        let state = bus_state(&mut bus_sessions, bus_id)?;
        state.send_set_name(device_id, &name).map_err(|e| {
            log_error!("Couldn't set device ID on {device_id_hex}: {e}!");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    tokio::time::sleep(Duration::from_millis(
        params
            .get("wait")
            .and_then(|w| w.parse::<u64>().ok())
            .unwrap_or(50),
    ))
    .await;

    Ok(Json(()))
}

async fn session_reboot(
    State(state): State<AppState>,
    Path((bus_id, device_id_hex)): Path<(u16, String)>,
    Query(params): Query<FxHashMap<String, bool>>,
) -> Result<Json<()>, StatusCode> {
    let device_id = session_hex(&device_id_hex)?;
    let bootloader = params.get("bootloader").copied().unwrap_or(false);
    {
        let mut bus_sessions = state.bus_sessions.lock();
        let state = bus_state(&mut bus_sessions, bus_id)?;
        state.send_reboot(device_id, bootloader).map_err(|e| {
            log_error!("Couldn't send reboot on {device_id_hex}: {e}!");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    Ok(Json(()))
}

fn session_hex(device_id_hex: &str) -> Result<u32, StatusCode> {
    u32::from_str_radix(&device_id_hex, 16).map_err(|_| {
        log_error!("Invalid session id {device_id_hex}");
        StatusCode::BAD_REQUEST
    })
}

fn pull_key<T: core::fmt::Debug, R, F: FnOnce(&T) -> Option<R>>(
    params: &FxHashMap<String, T>,
    key: &str,
    mapper: F,
) -> Result<R, StatusCode> {
    mapper(params.get(key).ok_or_else(|| {
        log_error!("Missing param key {key}");
        StatusCode::BAD_REQUEST
    })?)
    .ok_or_else(|| {
        log_error!("Param key {key}: invalid value {:?}", params.get(key));
        StatusCode::BAD_REQUEST
    })
}

fn bus_state<'a>(
    bus_sessions: &'a mut parking_lot::MutexGuard<'_, FxHashMap<u16, BusState>>,
    bus_id: u16,
) -> Result<&'a mut BusState, StatusCode> {
    bus_sessions.get_mut(&bus_id).ok_or_else(|| {
        log_error!("Bus {bus_id} not opened!");
        StatusCode::BAD_REQUEST
    })
}

// Generic OPTIONS handler for CORS preflight
//async fn options_handler() -> impl axum::response::IntoResponse {
//    (StatusCode::OK, "")
//}

pub async fn run_web_server(mut shutdown_pipe: watch::Receiver<bool>, fifocore: FIFOCore) {
    let state = AppState {
        fifocore,
        ota_clients: Default::default(),
        bus_sessions: Default::default(),
    };

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers([
            "User-Agent".parse().unwrap(),
            "Sec-Fetch-Mode".parse().unwrap(),
            "Referer".parse().unwrap(),
            "Origin".parse().unwrap(),
            "X-Arbitration".parse().unwrap(),
            "Access-Control-Request-Method".parse().unwrap(),
            "Access-Control-Request-Headers".parse().unwrap(),
            "Content-Type".parse().unwrap(),
            "Sec-Fetch-Site".parse().unwrap(),
            "Sec-Fetch-Dest".parse().unwrap(),
            "Accept".parse().unwrap(),
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ]);

    let mut app = Router::new()
        .route("/version", get(version_handler))
        .route("/banner", get(banner_handler))
        .route("/", get(configurator_handler))
        .route("/ws/{bus}", axum::routing::any(websocket_handler))
        .route("/buses", get(list_bus_handler))
        .route("/buses/open", get(open_bus_handler))
        // Open a bus for session monitoring. You need to explicitly open one to do anything else.
        .route("/sessions/open/{bus}", get(session_open_bus))
        // Close a session monitoring session
        .route("/sessions/close/{bus}", get(session_close_bus))
        // Send an enumerate packet (which forces _most_ devices to enumerate their serials, except really old Canandmags)
        .route("/sessions/{bus}/enumerate", get(session_enumerate_bus))
        // List detected devices
        .route("/sessions/{bus}/devices/list", get(session_list_devices))
        // Clear the currently detected devices list
        .route("/sessions/{bus}/devices/clear", get(session_clear_devices))
        .route(
            "/sessions/{bus}/devices/{device_id}/arbitrate",
            get(session_arb_device),
        )
        .route(
            "/sessions/{bus}/devices/{device_id}/blink",
            get(session_blink_device),
        )
        .route(
            "/sessions/{bus}/devices/{device_id}/set_id",
            get(session_set_id_device),
        )
        .route(
            "/sessions/{bus}/devices/{device_id}/fetch_setting",
            get(session_fetch_setting),
        )
        .route(
            "/sessions/{bus}/devices/{device_id}/set_name",
            get(session_set_name),
        )
        .route(
            "/sessions/{bus}/devices/{device_id}/reboot",
            get(session_reboot),
        )
        /*
        /sessions/{bus}/devices/{device_id}
         */
        .route("/ota/{bus}/{id}/start", post(crate::ota::ota_start_handler))
        .route(
            "/ota/{bus}/{id}/status",
            get(crate::ota::ota_status_handler),
        )
        .route("/ota/{bus}/{id}/abort", get(crate::ota::ota_abort_handler))
        .with_state(state.clone());
    //.route("/*_", options(options_handler))

    app = app.layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7244")
        .await
        .expect("Failed to bind to address");

    log_info!("Starting CANLink server on 0.0.0.0:7244");

    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        shutdown_pipe.wait_for(|f| *f).await.ok();
    });

    if let Err(e) = server.await {
        log_error!("Server error: {}", e);
    }
}
