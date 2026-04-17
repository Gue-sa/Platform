use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use diesel::prelude::*;
use dotenvy::dotenv;
use shared::{
    common::types::{DatabaseManagerError, DatabaseManagerResult, VoyageStatus},
    voyage_order::{VoyageOrder, VoyageOrderBody, VoyageOrderHeader},
};
use std::env;

use crate::database_manager::{
    models::{
        DestinationInsertionModel, DestinationQueryResult, VoyageOrderQueryResult,
        VoyageOrderVersionInsertionModel, VoyageOrderVersionQueryResult,
    },
    schema::{DESTINATIONS, ORDER_VERSIONS, VOYAGE_ORDERS},
};

pub struct DatabaseManager {
    connection: SqliteConnection,
}

impl DatabaseManager {
    pub fn init() -> DatabaseManagerResult<Self> {
        dotenv().is_ok();

        let database_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        let connection: SqliteConnection = SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        Ok(Self {
            connection: connection,
        })
    }

    pub fn add_destination(
        &mut self,
        name: &str,
        longitude: u16,
        latitude: u16,
    ) -> DatabaseManagerResult<()> {
        let new_destination: DestinationInsertionModel<'_> = DestinationInsertionModel {
            name: name,
            longitude: &longitude.into(),
            latitude: &latitude.into(),
        };

        diesel::insert_into(DESTINATIONS::table)
            .values(&new_destination)
            .returning(DestinationQueryResult::as_returning())
            .get_result::<DestinationQueryResult>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::InsertionError(e))?;

        Ok(())
    }

    pub fn add_voyage_order_version(
        &mut self,
        order_id: i32,
        version_number: u8,
        destination: String,
        eta_month: u8,
        eta_day: u8,
        eta_hour: u8,
        eta_minute: u8,
        cargo_type: u8,
        speed_profile: u8,
    ) -> DatabaseManagerResult<()> {
        let destination_id: i32 = DESTINATIONS::table
            .filter(DESTINATIONS::name.eq(destination))
            .select(DESTINATIONS::id)
            .first::<i32>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?;

        let new_voyage_order_version: VoyageOrderVersionInsertionModel<'_> =
            VoyageOrderVersionInsertionModel {
                version_number: &version_number.into(),
                order_id: &order_id,
                destination_id: &destination_id,
                eta: &NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(0, eta_month.into(), eta_day.into()).unwrap(),
                    NaiveTime::from_hms_opt(eta_hour.into(), eta_minute.into(), 0).unwrap(),
                ),
                cargo_type: &cargo_type.into(),
                speed_profile: &speed_profile.into(),
            };

        diesel::insert_into(ORDER_VERSIONS::table)
            .values(&new_voyage_order_version)
            .returning(VoyageOrderVersionQueryResult::as_returning())
            .get_result::<VoyageOrderVersionQueryResult>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::InsertionError(e))?;

        Ok(())
    }

    pub fn add_voyage_order(
        &mut self,
        destination: String,
        eta_month: u8,
        eta_day: u8,
        eta_hour: u8,
        eta_minute: u8,
        cargo_type: u8,
        speed_profile: u8,
    ) -> DatabaseManagerResult<VoyageOrder> {
        let order_id: i32 = diesel::insert_into(VOYAGE_ORDERS::table)
            .default_values()
            .returning(VOYAGE_ORDERS::id)
            .get_result::<i32>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::InsertionError(e))?;

        self.add_voyage_order_version(
            order_id.clone(),
            0,
            destination.clone(),
            eta_month,
            eta_day,
            eta_hour,
            eta_minute,
            cargo_type,
            speed_profile,
        )?;

        let destination_info: DestinationQueryResult = DESTINATIONS::table
            .filter(DESTINATIONS::name.eq(destination.clone()))
            .select(DestinationQueryResult::as_returning())
            .first::<DestinationQueryResult>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?;

        Ok(VoyageOrder {
            header: VoyageOrderHeader {
                id: order_id as u16,
                version: 0,
            },
            body: VoyageOrderBody {
                destination: destination,
                destination_position: (
                    destination_info.longitude as u16,
                    destination_info.latitude as u16,
                ),
                eta_month: eta_month,
                eta_day: eta_day,
                eta_hour: eta_hour,
                eta_minute: eta_minute,
                cargo_type: cargo_type,
                speed_profile: speed_profile,
            },
        })
    }

    pub fn update_destination(&mut self) -> DatabaseManagerResult<()> {
        todo!()
    }

    pub fn update_voyage_order(&mut self) -> DatabaseManagerResult<()> {
        todo!()
    }

    pub fn delete_destination(&mut self) -> DatabaseManagerResult<()> {
        todo!()
    }

    pub fn get_destinations(
        &mut self,
        id: Option<i32>,
        name: Option<String>,
        longitude: Option<u16>,
        latitude: Option<u16>,
    ) -> DatabaseManagerResult<Box<[DestinationQueryResult]>> {
        let mut query = DESTINATIONS::table.into_boxed();

        if let Some(v) = id {
            query = query.filter(DESTINATIONS::id.eq(v));
        };

        if let Some(v) = name {
            query = query.filter(DESTINATIONS::name.eq(v));
        };

        if let Some(v) = longitude {
            query = query.filter(DESTINATIONS::longitude.eq(v as i32));
        };

        if let Some(v) = latitude {
            query = query.filter(DESTINATIONS::latitude.eq(v as i32));
        };

        Ok(query
            .load::<DestinationQueryResult>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?
            .into_boxed_slice())
    }

    pub fn get_voyage_order_versions(
        &mut self,
        voyage_order_id: Option<u16>,
        version_number: Option<u8>,
    ) -> DatabaseManagerResult<Box<[VoyageOrderVersionQueryResult]>> {
        let mut query = ORDER_VERSIONS::table.into_boxed();

        if let Some(v) = voyage_order_id {
            query = query.filter(ORDER_VERSIONS::order_id.eq(v as i32));
        };

        if let Some(v) = version_number {
            query = query.filter(ORDER_VERSIONS::version_number.eq(v as i32));
        };

        Ok(query
            .load::<VoyageOrderVersionQueryResult>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?
            .into_boxed_slice())
    }

    pub fn get_voyage_orders(
        &mut self,
        voyage_order_id: Option<u16>,
        status: Option<VoyageStatus>,
        executant: Option<u32>,
    ) -> DatabaseManagerResult<Box<[VoyageOrder]>> {
        let mut query = VOYAGE_ORDERS::table
            .inner_join(
                ORDER_VERSIONS::table
                    .on(ORDER_VERSIONS::order_id.eq(VOYAGE_ORDERS::id).and(
                        ORDER_VERSIONS::version_number.eq(VOYAGE_ORDERS::current_version_number),
                    ))
                    .inner_join(DESTINATIONS::table),
            )
            .into_boxed::<'_, diesel::sqlite::Sqlite>();

        if let Some(v) = voyage_order_id {
            query = query.filter(VOYAGE_ORDERS::id.eq(v as i32));
        };

        if let Some(v) = status {
            query = query.filter(VOYAGE_ORDERS::status.eq(Into::<u8>::into(v) as i32));
        };

        if let Some(v) = executant {
            query = query.filter(VOYAGE_ORDERS::executant.eq(v as i32));
        };

        let results: Vec<(
            VoyageOrderQueryResult,
            VoyageOrderVersionQueryResult,
            DestinationQueryResult,
        )> = query
            .select((
                VoyageOrderQueryResult::as_select(),
                VoyageOrderVersionQueryResult::as_select(),
                DestinationQueryResult::as_select(),
            ))
            .load::<(
                VoyageOrderQueryResult,
                VoyageOrderVersionQueryResult,
                DestinationQueryResult,
            )>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?;

        let voyage_orders: Box<[VoyageOrder]> = results
            .into_iter()
            .map(|(o, v, d)| VoyageOrder {
                header: VoyageOrderHeader {
                    id: o.id as u16,
                    version: o.current_version_number as u8,
                },
                body: VoyageOrderBody {
                    destination: d.name,
                    destination_position: (d.longitude as u16, d.latitude as u16),
                    eta_month: v.eta.month() as u8,
                    eta_day: v.eta.day() as u8,
                    eta_hour: v.eta.hour() as u8,
                    eta_minute: v.eta.minute() as u8,
                    cargo_type: v.cargo_type as u8,
                    speed_profile: v.speed_profile as u8,
                },
            })
            .collect();

        Ok(voyage_orders)
    }

    pub fn get_voyage_order_versions_count(&mut self) -> DatabaseManagerResult<()> {
        todo!()
    }

    pub fn get_voyage_orders_count(&mut self) -> DatabaseManagerResult<()> {
        todo!()
    }

    pub fn has_version(&mut self) -> DatabaseManagerResult<()> {
        todo!()
    }

    pub fn is_current_version(&mut self) -> DatabaseManagerResult<()> {
        todo!()
    }

    pub fn update_voyage_order_status(&mut self) -> DatabaseManagerResult<()> {
        todo!()
    }

    pub fn assign_voyage_order(&mut self) -> DatabaseManagerResult<()> {
        todo!()
    }

    pub fn get_unassigned_voyage_orders(&mut self) -> DatabaseManagerResult<()> {
        todo!()
    }

    pub fn get_assigned_voyage_orders(&mut self) -> DatabaseManagerResult<()> {
        todo!()
    }
}
