    CREATE TABLE DESTINATIONS (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        name VARCHAR(20) NOT NULL UNIQUE,
        longitude INTEGER NOT NULL,
        latitude INTEGER NOT NULL,
        UNIQUE (longitude, latitude)
    );