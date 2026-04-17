use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::database_manager::schema::{DESTINATIONS, ORDER_VERSIONS, VOYAGE_ORDERS};

#[derive(Queryable, Selectable)]
#[diesel(table_name = DESTINATIONS)]
#[diesel(belongs_to(ORDER_VERSIONS))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DestinationQueryResult {
    pub id: i32,
    pub name: String,
    pub longitude: i32,
    pub latitude: i32,
}

#[derive(Insertable)]
#[diesel(table_name = DESTINATIONS)]
pub struct DestinationInsertionModel<'a> {
    pub name: &'a str,
    pub longitude: &'a i32,
    pub latitude: &'a i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = VOYAGE_ORDERS)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct VoyageOrderQueryResult {
    pub id: i32,
    pub creation_date: NaiveDateTime,
    pub status: i32,
    pub executant: Option<i32>,
    pub current_version_number: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = ORDER_VERSIONS)]
#[diesel(belongs_to(VOYAGE_ORDERS))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct VoyageOrderVersionQueryResult {
    pub id: i32,
    pub version_number: i32,
    pub creation_date: NaiveDateTime,
    pub order_id: i32,
    pub destination_id: i32,
    pub eta: NaiveDateTime,
    pub cargo_type: i32,
    pub speed_profile: i32,
}

#[derive(Insertable)]
#[diesel(table_name = ORDER_VERSIONS)]
pub struct VoyageOrderVersionInsertionModel<'a> {
    pub version_number: &'a i32,
    pub order_id: &'a i32,
    pub destination_id: &'a i32,
    pub eta: &'a NaiveDateTime,
    pub cargo_type: &'a i32,
    pub speed_profile: &'a i32,
}
