// @generated automatically by Diesel CLI.

diesel::table! {
    DESTINATIONS (id) {
        id -> Integer,
        name -> Text,
        longitude -> Integer,
        latitude -> Integer,
    }
}

diesel::table! {
    ORDER_VERSIONS (id) {
        id -> Integer,
        version_number -> Integer,
        creation_date -> Timestamp,
        order_id -> Integer,
        destination_id -> Integer,
        eta -> Timestamp,
        cargo_type -> Integer,
        speed_profile -> Integer,
    }
}

diesel::table! {
    VOYAGE_ORDERS (id) {
        id -> Integer,
        creation_date -> Timestamp,
        status -> Integer,
        executant -> Nullable<Integer>,
        current_version_number -> Integer,
    }
}

diesel::joinable!(ORDER_VERSIONS -> DESTINATIONS (destination_id));

diesel::allow_tables_to_appear_in_same_query!(DESTINATIONS, ORDER_VERSIONS, VOYAGE_ORDERS,);
