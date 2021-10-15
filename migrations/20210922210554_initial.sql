ALTER DATABASE CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;
CREATE TABLE character_public_info (
    character_id BIGINT unsigned PRIMARY KEY UNIQUE NOT NULL,
    character_name TEXT UNIQUE NOT NULL,
    alliance_id BIGINT unsigned,
    birthday TEXT,
    corporation_id BIGINT unsigned NOT NULL,
    faction_id BIGINT unsigned
);

CREATE TABLE esi_categories (
    category_id BIGINT unsigned PRIMARY KEY UNIQUE NOT NULL,
    category_name TEXT NOT NULL
);

CREATE TABLE esi_groups (
    group_id BIGINT unsigned PRIMARY KEY UNIQUE NOT NULL,
    group_name TEXT NOT NULL,
    category_id BIGINT unsigned NOT NULL,
    FOREIGN KEY (category_id) REFERENCES esi_categories (category_id) ON DELETE CASCADE
);

CREATE TABLE esi_types (
    type_id BIGINT unsigned PRIMARY KEY UNIQUE NOT NULL,
    type_name TEXT NOT NULL,
    description TEXT NOT NULL,
    mass FLOAT,
    group_id BIGINT unsigned NOT NULL,
    FOREIGN KEY (group_id) REFERENCES esi_groups (group_id) ON DELETE CASCADE
);

CREATE TABLE killmails (
    killmail_id BIGINT unsigned PRIMARY KEY UNIQUE NOT NULL,
    killmail_time DATETIME NOT NULL,
    solar_system_id BIGINT unsigned NOT NULL
);