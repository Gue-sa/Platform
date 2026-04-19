use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use serde::Deserialize;
use shared::{boat_info::BoatInfo, boats_registry::BoatsInfoRegistry};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::database_manager::{
    manager::DatabaseManager,
    models::{DestinationQueryResult, VoyageOrderQueryResult, VoyageOrderVersionQueryResult},
};

#[derive(Deserialize)]
pub struct GetVoyageOrderVersionsFilterParams {
    order_id: Option<i32>,
    version_number: Option<i32>,
}

pub struct DatabaseApiSharedState {
    pub database_manager: Arc<Mutex<DatabaseManager>>,
    pub boats_registry: Arc<BoatsInfoRegistry>,
}

pub struct DatabaseApi {
    pub state: Arc<DatabaseApiSharedState>,
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
            //.route("/get_boat_info", get(Self::get_boat_info))
            //.route("/get_statistics", get(Self::get_statistics))
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
        let mut manager: std::sync::MutexGuard<'_, DatabaseManager> = shared_state.database_manager.lock().unwrap();

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
        let mut manager: std::sync::MutexGuard<'_, DatabaseManager> = shared_state.database_manager.lock().unwrap();

        let results: Box<[(VoyageOrderVersionQueryResult, DestinationQueryResult)]> = manager
            .get_voyage_order_versions(params.order_id, params.version_number)
            .unwrap();

        Json(results)
    }

    async fn get_boats_list(State(shared_state): State<Arc<DatabaseApiSharedState>>) -> Json<Box<[(u32, BoatInfo)]>> {
        Json(shared_state.boats_registry.export())
    }
}
