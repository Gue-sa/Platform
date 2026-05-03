use crate::database_manager::{
    models::{
        DestinationInsertionModel, DestinationQueryResult, VoyageOrderQueryResult,
        VoyageOrderVersionInsertionModel, VoyageOrderVersionQueryResult,
    },
    schema::{DESTINATIONS, ORDER_VERSIONS, VOYAGE_ORDERS},
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use shared::{
    common::{
        errors::{DatabaseManagerError, DatabaseManagerResult},
        types::VoyageStatus,
    },
    voyage_order::{VoyageOrder, VoyageOrderBody, VoyageOrderHeader},
};
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct DatabaseManager {
    connection: SqliteConnection,
}

impl DatabaseManager {
    pub fn init() -> DatabaseManagerResult<Self> {
        let mut path = env::current_exe().expect("Impossible de trouver le chemin de l'exécutable");

        path.pop();
        path.push("harbourmaster_database.db");

        let db_url = path.to_str().expect("Chemin invalide").to_string();

        let mut connection = SqliteConnection::establish(&db_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", db_url));

        connection
            .run_pending_migrations(MIGRATIONS)
            .map_err(|e| DatabaseManagerError::QueryError(diesel::result::Error::NotFound))?;

        Ok(Self {
            connection: connection,
        })
    }

    pub fn add_destination(&mut self, name: &str, lon: i32, lat: i32) -> DatabaseManagerResult<()> {
        let new_dest: DestinationInsertionModel<'_> = DestinationInsertionModel {
            name: name,
            longitude: &lon,
            latitude: &lat,
        };

        diesel::insert_into(DESTINATIONS::table)
            .values(&new_dest)
            .returning(DestinationQueryResult::as_returning())
            .get_result::<DestinationQueryResult>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::InsertionError(e))?;

        Ok(())
    }

    pub fn add_voyage_order_version(
        &mut self,
        order_id: i32,
        version_nbr: i32,
        dest_id: i32,
        eta_month: i32,
        eta_day: i32,
        eta_hour: i32,
        eta_min: i32,
        cargo_type: i32,
        speed_profile: i32,
    ) -> DatabaseManagerResult<()> {
        let eta = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(0, eta_month as u32, eta_day as u32)
                .ok_or(DatabaseManagerError::InvalidNaiveDate)?,
            NaiveTime::from_hms_opt(eta_hour as u32, eta_min as u32, 0)
                .ok_or(DatabaseManagerError::InvalidNaiveDate)?,
        );

        let new_voyage_order_version: VoyageOrderVersionInsertionModel<'_> =
            VoyageOrderVersionInsertionModel {
                version_number: &version_nbr,
                order_id: &order_id,
                destination_id: &dest_id,
                eta: &eta,
                cargo_type: &cargo_type,
                speed_profile: &speed_profile,
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
        dest_id: i32,
        eta_month: i32,
        eta_day: i32,
        eta_hour: i32,
        eta_min: i32,
        cargo_type: i32,
        speed_profile: i32,
    ) -> DatabaseManagerResult<VoyageOrder> {
        let order_id = diesel::insert_into(VOYAGE_ORDERS::table)
            .default_values()
            .returning(VOYAGE_ORDERS::id)
            .get_result::<i32>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::InsertionError(e))?;

        let dest_name = DESTINATIONS::table
            .filter(DESTINATIONS::id.eq(dest_id))
            .select(DESTINATIONS::name)
            .first::<String>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?;

        self.add_voyage_order_version(
            order_id,
            0,
            dest_id,
            eta_month,
            eta_day,
            eta_hour,
            eta_min,
            cargo_type,
            speed_profile,
        )?;

        let destination_info = DESTINATIONS::table
            .filter(DESTINATIONS::name.eq(&dest_name))
            .select(DestinationQueryResult::as_returning())
            .first::<DestinationQueryResult>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?;

        let header = VoyageOrderHeader::from_data(order_id as u16, 0);
        let body = VoyageOrderBody::from_data(
            &dest_name,
            (
                destination_info.longitude as u16,
                destination_info.latitude as u16,
            ),
            eta_month as u8,
            eta_day as u8,
            eta_hour as u8,
            eta_min as u8,
            cargo_type as u8,
            speed_profile as u8,
        );

        Ok(VoyageOrder::from_components(&header, &body))
    }

    pub fn get_destinations(
        &mut self,
        id: Option<i32>,
        name: Option<&str>,
        lon: Option<i32>,
        lat: Option<i32>,
    ) -> DatabaseManagerResult<Box<[DestinationQueryResult]>> {
        let mut query = DESTINATIONS::table.into_boxed();

        if let Some(v) = id {
            query = query.filter(DESTINATIONS::id.eq(v));
        };

        if let Some(v) = name {
            query = query.filter(DESTINATIONS::name.eq(v));
        };

        if let Some(v) = lon {
            query = query.filter(DESTINATIONS::longitude.eq(v));
        };

        if let Some(v) = lat {
            query = query.filter(DESTINATIONS::latitude.eq(v));
        };

        Ok(query
            .load::<DestinationQueryResult>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?
            .into_boxed_slice())
    }

    pub fn get_voyage_order_versions(
        &mut self,
        voyage_order_id: Option<i32>,
        version_nbr: Option<i32>,
    ) -> DatabaseManagerResult<Box<[(VoyageOrderVersionQueryResult, DestinationQueryResult)]>> {
        let mut query = ORDER_VERSIONS::table
            .inner_join(DESTINATIONS::table)
            .into_boxed();

        if let Some(v) = voyage_order_id {
            query = query.filter(ORDER_VERSIONS::order_id.eq(v));
        };

        if let Some(v) = version_nbr {
            query = query.filter(ORDER_VERSIONS::version_number.eq(v));
        };

        Ok(query
            .load::<(VoyageOrderVersionQueryResult, DestinationQueryResult)>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?
            .into_boxed_slice())
    }

    pub fn get_voyage_orders(
        &mut self,
        voyage_order_id: Option<i32>,
        status: Option<VoyageStatus>,
        executant: Option<i32>,
    ) -> DatabaseManagerResult<
        Box<
            [(
                VoyageOrderQueryResult,
                VoyageOrderVersionQueryResult,
                DestinationQueryResult,
            )],
        >,
    > {
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
            query = query.filter(VOYAGE_ORDERS::id.eq(v));
        };

        if let Some(v) = status {
            query = query.filter(VOYAGE_ORDERS::status.eq(v as u8 as i32));
        };

        if let Some(v) = executant {
            query = query.filter(VOYAGE_ORDERS::executant.eq(v));
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

        Ok(results.into_boxed_slice())
    }

    pub fn get_voyage_order_versions_count(
        &mut self,
        voyage_order_id: i32,
    ) -> DatabaseManagerResult<usize> {
        let count = ORDER_VERSIONS::table
            .filter(ORDER_VERSIONS::order_id.eq(voyage_order_id))
            .count()
            .get_result::<i64>(&mut self.connection)
            .map_err(DatabaseManagerError::QueryError)?;

        Ok(count as usize)
    }

    pub fn get_voyage_order_rev_ver(
        &mut self,
        order_id: i32,
    ) -> DatabaseManagerResult<Option<VoyageOrderVersionQueryResult>> {
        let current_version = VOYAGE_ORDERS::table
            .filter(VOYAGE_ORDERS::id.eq(order_id))
            .select(VOYAGE_ORDERS::current_version_number)
            .first::<i32>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?;

        if self.has_version(order_id, current_version + 1)? {
            let rev_ver = ORDER_VERSIONS::table
                .filter(
                    ORDER_VERSIONS::order_id
                        .eq(order_id)
                        .and(ORDER_VERSIONS::version_number.eq(current_version + 1)),
                )
                .select(VoyageOrderVersionQueryResult::as_returning())
                .first::<VoyageOrderVersionQueryResult>(&mut self.connection)
                .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?;

            Ok(Some(rev_ver))
        } else {
            Ok(None)
        }
    }

    pub fn get_orders_count(&mut self) -> DatabaseManagerResult<usize> {
        let count = VOYAGE_ORDERS::table
            .count()
            .get_result::<i64>(&mut self.connection)
            .map_err(DatabaseManagerError::QueryError)?;

        Ok(count as usize)
    }

    pub fn get_free_orders_count(&mut self) -> DatabaseManagerResult<usize> {
        let count = VOYAGE_ORDERS::table
            .filter(VOYAGE_ORDERS::executant.is_null())
            .count()
            .get_result::<i64>(&mut self.connection)
            .map_err(DatabaseManagerError::QueryError)?;

        Ok(count as usize)
    }

    pub fn has_version(
        &mut self,
        voyage_order_id: i32,
        version: i32,
    ) -> DatabaseManagerResult<bool> {
        let count = ORDER_VERSIONS::table
            .filter(
                ORDER_VERSIONS::order_id
                    .eq(voyage_order_id)
                    .and(ORDER_VERSIONS::version_number.eq(version)),
            ) // On filtre par l'ID de l'ordre
            .count()
            .get_result::<i64>(&mut self.connection)
            .map_err(DatabaseManagerError::QueryError)?;

        Ok(count == 1)
    }

    pub fn is_current_version(
        &mut self,
        voyage_order_id: i32,
        version: i32,
    ) -> DatabaseManagerResult<bool> {
        let current_version = VOYAGE_ORDERS::table
            .filter(VOYAGE_ORDERS::id.eq(voyage_order_id))
            .select(VOYAGE_ORDERS::current_version_number)
            .get_result::<i32>(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::QueryError(e))?;

        Ok(current_version == version)
    }

    pub fn update_voyage_order_status(
        &mut self,
        voyage_order_id: i32,
        status: VoyageStatus,
    ) -> DatabaseManagerResult<()> {
        diesel::update(VOYAGE_ORDERS::table.filter(VOYAGE_ORDERS::id.eq(voyage_order_id)))
            .set(VOYAGE_ORDERS::status.eq(status as u8 as i32))
            .execute(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::UpdateError(e))?;

        Ok(())
    }

    pub fn assign_voyage_order(
        &mut self,
        voyage_order_id: i32,
        mmsi: i32,
    ) -> DatabaseManagerResult<()> {
        diesel::update(VOYAGE_ORDERS::table.filter(VOYAGE_ORDERS::id.eq(voyage_order_id)))
            .set(VOYAGE_ORDERS::executant.eq(mmsi))
            .execute(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::UpdateError(e))?;

        Ok(())
    }

    pub fn update_voyage_order_version(
        &mut self,
        voyage_order_id: i32,
        version_nbr: i32,
    ) -> DatabaseManagerResult<()> {
        if self.has_version(voyage_order_id, version_nbr)? {
            diesel::update(VOYAGE_ORDERS::table.filter(VOYAGE_ORDERS::id.eq(voyage_order_id)))
                .set(VOYAGE_ORDERS::current_version_number.eq(version_nbr))
                .execute(&mut self.connection)
                .map_err(|e: diesel::result::Error| DatabaseManagerError::UpdateError(e))?;

            Ok(())
        } else {
            return Err(DatabaseManagerError::UpdateError(
                diesel::result::Error::NotFound,
            ));
        }
    }

    pub fn delete_destination(&mut self, dest_id: i32) -> DatabaseManagerResult<()> {
        diesel::delete(DESTINATIONS::table.filter(DESTINATIONS::id.eq(dest_id)))
            .execute(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::DeletionError(e))?;

        Ok(())
    }

    pub fn delete_voyage_order_version(
        &mut self,
        voyage_order_id: i32,
        version_nbr: i32,
    ) -> DatabaseManagerResult<()> {
        diesel::delete(
            ORDER_VERSIONS::table.filter(
                ORDER_VERSIONS::order_id
                    .eq(voyage_order_id)
                    .and(ORDER_VERSIONS::version_number.eq(version_nbr)),
            ),
        )
        .execute(&mut self.connection)
        .map_err(|e: diesel::result::Error| DatabaseManagerError::DeletionError(e))?;

        Ok(())
    }

    pub fn delete_voyage_order_versions(
        &mut self,
        voyage_order_id: i32,
    ) -> DatabaseManagerResult<()> {
        diesel::delete(ORDER_VERSIONS::table.filter(ORDER_VERSIONS::order_id.eq(voyage_order_id)))
            .execute(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::DeletionError(e))?;

        Ok(())
    }

    pub fn delete_voyage_order(&mut self, voyage_order_id: i32) -> DatabaseManagerResult<()> {
        self.delete_voyage_order_versions(voyage_order_id)?;

        diesel::delete(VOYAGE_ORDERS::table.filter(VOYAGE_ORDERS::id.eq(voyage_order_id)))
            .execute(&mut self.connection)
            .map_err(|e: diesel::result::Error| DatabaseManagerError::DeletionError(e))?;

        Ok(())
    }
}
