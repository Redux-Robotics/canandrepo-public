use axum::response::IntoResponse;
use axum::response::Json;
use fifocore::{FIFOCore, error::Error};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ListBuses {
    pub buses: Vec<BusEntry>,
    pub time_now: i64,
    pub time_mono: i64,
}

#[derive(Debug, Serialize)]
pub struct BusEntry {
    pub id: u16,
    pub params: String,
    pub id_cache: fifocore::backends::IdCache,
}

pub fn handle_list_bus(cdn: &FIFOCore) -> ListBuses {
    cdn.with_buses(|buses| ListBuses {
        buses: buses
            .iter()
            .map(|(&id, ent)| BusEntry {
                id,
                params: ent.params().to_string(),
                id_cache: ent.id_cache(),
            })
            .collect(),
        time_now: fifocore::timebase::now_us(),
        time_mono: fifocore::timebase::monotonic_us(),
    })
}

#[derive(Debug, Serialize)]
pub struct BusOpenSuccess {
    pub id: u16,
    pub params: String,
}

#[derive(Debug, Serialize)]
pub struct FIFOCoreError {
    pub error_id: i32,
    pub reason: String,
}

impl From<Error> for FIFOCoreError {
    fn from(value: Error) -> Self {
        Self {
            error_id: value as i32,
            reason: value.message().to_owned(),
        }
    }
}

pub fn handle_open_bus(fifocore: &FIFOCore, bus_name: &str) -> axum::response::Response {
    match fifocore.open_or_get_bus(&bus_name) {
        Ok(id) => Json(BusOpenSuccess {
            id,
            params: bus_name.to_owned(),
        })
        .into_response(),
        Err(e) => Json(FIFOCoreError::from(e)).into_response(),
    }
}
