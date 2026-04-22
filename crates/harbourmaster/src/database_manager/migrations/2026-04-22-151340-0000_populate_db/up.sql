INSERT INTO DESTINATIONS (name, longitude, latitude)
VALUES ("DEST_TEST", 1000, 500);

INSERT INTO VOYAGE_ORDERS DEFAULT
VALUES;

INSERT INTO ORDER_VERSIONS (
        order_id,
        destination_id,
        eta,
        cargo_type,
        speed_profile
    )
VALUES (
        last_insert_rowid(),
        (
            SELECT id
            FROM DESTINATIONS
            WHERE name = "DEST_TEST"
            ORDER BY id DESC
            LIMIT 1
        ), "2026-04-22 17:23:19", 0, 0
    );