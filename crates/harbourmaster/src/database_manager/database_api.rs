use crate::database_manager::{
    manager::DatabaseManager,
    models::{DestinationQueryResult, VoyageOrderQueryResult, VoyageOrderVersionQueryResult},
};
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use shared::{boat_info::BoatInfo, boats_registry::BoatsInfoRegistry, common::types::LogEvent};
use std::sync::{Arc, Mutex, mpsc::Sender};
use tokio::{net::TcpListener, task::JoinHandle};
use tower_http::cors::CorsLayer;

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

#[derive(Serialize, Debug)]
pub struct StatisticsResult {
    boats_nbr: usize,
    active_boats_nbr: usize,
    unresponding_boats_nbr: usize,
    orders_nbr: usize,
    free_orders_nbr: usize,
}

pub struct DatabaseApiSharedState {
    database_manager: Arc<Mutex<DatabaseManager>>,
    boats_registry: Arc<BoatsInfoRegistry>,
    logs_cli_tx: Sender<LogEvent>,
}

pub struct DatabaseApi {
    state: Arc<DatabaseApiSharedState>,
}

impl DatabaseApiSharedState {
    fn logs_cli_tx(&self) -> Sender<LogEvent> {
        self.logs_cli_tx.clone()
    }
}

impl DatabaseApi {
    pub fn init(
        database_manager: Arc<Mutex<DatabaseManager>>,
        boats_reg: Arc<BoatsInfoRegistry>,
        cli_tx: Sender<LogEvent>,
    ) -> Self {
        Self {
            state: Arc::new(DatabaseApiSharedState {
                database_manager: database_manager,
                boats_registry: boats_reg,
                logs_cli_tx: cli_tx,
            }),
        }
    }

    pub async fn start(self) -> JoinHandle<()> {
        self.state
            .logs_cli_tx()
            .send(LogEvent::System("Lancement de l'API armateur...".yellow()));

        tokio::spawn(async move {
            let api = Router::new()
                .route("/", get(Self::welcome))
                .route("/get_voyage_orders", get(Self::get_voyage_orders))
                .route(
                    "/get_voyage_order_versions",
                    get(Self::get_voyage_order_versions),
                )
                .route("/get_boats_list", get(Self::get_boats_list))
                .route("/get_destinations", get(Self::get_destinations))
                .route("/get_statistics", get(Self::get_statistics))
                .route("/add_voyage_order", post(Self::create_voyage_order))
                .with_state(self.state.clone())
                .layer(CorsLayer::permissive());

            let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();

            axum::serve(listener, api).await.unwrap();
        })
    }

    async fn welcome() -> String {
        "Bonjour et bienvenue sur l'API armateur ! Voici la liste des commandes actuellement implémentées:\n- /get_voyage_orders\n- /get_voyage_orders_versions\n- /get_boats_list\n- /get_voyage_destinations\n- /get_statistics\n- /add_voyage_order".to_string()
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
        let mut db_manager: std::sync::MutexGuard<'_, DatabaseManager> =
            shared_state.database_manager.lock().unwrap();

        let results: Box<[DestinationQueryResult]> =
            db_manager.get_destinations(None, None, None, None).unwrap();

        Json(results)
    }

    async fn get_statistics(
        State(shared_state): State<Arc<DatabaseApiSharedState>>,
    ) -> Json<StatisticsResult> {
        let mut db_manager: std::sync::MutexGuard<'_, DatabaseManager> =
            shared_state.database_manager.lock().unwrap();

        let boats_nbr = shared_state.boats_registry.length();
        let active_boats_nbr = shared_state.boats_registry.count_active_boats();
        let orders_nbr = db_manager.get_orders_count().unwrap();
        let free_orders_nbr = db_manager.get_free_orders_count().unwrap();

        let res = StatisticsResult {
            boats_nbr: boats_nbr,
            active_boats_nbr: active_boats_nbr,
            orders_nbr: orders_nbr,
            free_orders_nbr: free_orders_nbr,
            unresponding_boats_nbr: 0,
        };

        Json(res)
    }

    async fn create_voyage_order(
        State(shared_state): State<Arc<DatabaseApiSharedState>>,
        Json(payload): Json<CreateVoyageOrderPayload>,
    ) -> () {
        let mut db_manager: std::sync::MutexGuard<'_, DatabaseManager> =
            shared_state.database_manager.lock().unwrap();

        db_manager.add_voyage_order(
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
