use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use shared::{boat_info::BoatInfo, boats_registry::BoatsInfoRegistry};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::database_manager::{
    manager::DatabaseManager,
    models::{DestinationQueryResult, VoyageOrderQueryResult, VoyageOrderVersionQueryResult},
};

#[derive(Deserialize, Debug)]
pub struct GetVoyageOrderVersionsFilterParams {
    order_id: Option<i32>,
    version_number: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct CreateVoyageOrderPayload {
    destination_id: i32,
    ship_type: u8,
    speed_profile: u8,
}

pub struct DatabaseApiSharedState {
    database_manager: Arc<Mutex<DatabaseManager>>,
    boats_registry: Arc<BoatsInfoRegistry>,
}

pub struct DatabaseApi {
    state: Arc<DatabaseApiSharedState>,
}

impl DatabaseApi {
    pub fn init(
        database_manager: Arc<Mutex<DatabaseManager>>,
        boats_registry: Arc<BoatsInfoRegistry>,
    ) -> Self {
        Self {
            state: Arc::new(DatabaseApiSharedState {
                database_manager: database_manager,
                boats_registry: boats_registry,
            }),
        }
    }

    pub async fn start(self) -> () {
        let api: Router = Router::new()
            .route("/get_voyage_orders", get(Self::get_voyage_orders))
            .route(
                "/get_voyage_order_versions",
                get(Self::get_voyage_order_versions),
            )
            .route("/get_boats_list", get(Self::get_boats_list))
            .route("/get_destinations", get(Self::get_destinations))
            //.route("/get_statistics", get(Self::get_statistics))
            .route("/add_voyage_order", post(Self::create_voyage_order))
            .with_state(self.state.clone())
            .layer(CorsLayer::permissive());

        let listener: TcpListener = TcpListener::bind("0.0.0.0:8000").await.unwrap();

        axum::serve(listener, api).await.unwrap();
    }

    async fn get_voyage_orders(
        State(shared_state): State<Arc<DatabaseApiSharedState>>,
    ) -> Json<
        Box<
            [(
                VoyageOrderQueryResult,
                VoyageOrderVersionQueryResult,
                DestinationQueryResult,
            )],
        >,
    > {
        let mut manager: std::sync::MutexGuard<'_, DatabaseManager> =
            shared_state.database_manager.lock().unwrap();

        let results: Box<
            [(
                VoyageOrderQueryResult,
                VoyageOrderVersionQueryResult,
                DestinationQueryResult,
            )],
        > = manager.get_voyage_orders(None, None, None).unwrap();

        Json(results)
    }

    async fn get_voyage_order_versions(
        Query(params): Query<GetVoyageOrderVersionsFilterParams>,
        State(shared_state): State<Arc<DatabaseApiSharedState>>,
    ) -> Json<Box<[(VoyageOrderVersionQueryResult, DestinationQueryResult)]>> {
        let mut manager: std::sync::MutexGuard<'_, DatabaseManager> =
            shared_state.database_manager.lock().unwrap();

        let results: Box<[(VoyageOrderVersionQueryResult, DestinationQueryResult)]> = manager
            .get_voyage_order_versions(params.order_id, params.version_number)
            .unwrap();

        Json(results)
    }

    async fn get_boats_list(
        State(shared_state): State<Arc<DatabaseApiSharedState>>,
    ) -> Json<Box<[(u32, BoatInfo)]>> {
        Json(shared_state.boats_registry.export())
    }

    async fn get_destinations(
        State(shared_state): State<Arc<DatabaseApiSharedState>>,
    ) -> Json<Box<[DestinationQueryResult]>> {
        let mut manager: std::sync::MutexGuard<'_, DatabaseManager> =
            shared_state.database_manager.lock().unwrap();

        let results: Box<[DestinationQueryResult]> =
            manager.get_destinations(None, None, None, None).unwrap();

        Json(results)
    }

    async fn create_voyage_order(
        State(shared_state): State<Arc<DatabaseApiSharedState>>,
        Json(payload): Json<CreateVoyageOrderPayload>,
    ) -> () {
        let mut manager: std::sync::MutexGuard<'_, DatabaseManager> =
            shared_state.database_manager.lock().unwrap();

        manager.add_voyage_order(
            payload.destination_id.into(),
            1,
            1,
            0,
            0,
            payload.ship_type.into(),
            payload.speed_profile.into(),
        );
    }
}
