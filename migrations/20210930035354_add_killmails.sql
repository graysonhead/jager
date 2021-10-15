CREATE TABLE victims (
    victim_id BIGINT unsigned PRIMARY KEY UNIQUE NOT NULL AUTO_INCREMENT,
    alliance_id BIGINT unsigned,
    character_id BIGINT unsigned,
    corporation_id BIGINT unsigned,
    faction_id BIGINT unsigned,
    damage_taken BIGINT unsigned NOT NULL,
    ship_type_id BIGINT unsigned NOT NULL,
    killmail_id BIGINT unsigned NOT NULL,
    FOREIGN KEY (killmail_id) REFERENCES killmails (killmail_id) ON DELETE CASCADE,
    FOREIGN KEY (ship_type_id) REFERENCES esi_types (type_id),
    FOREIGN KEY (character_id) REFERENCES character_public_info (character_id)
);
CREATE TABLE attackers (
    attacker_id BIGINT unsigned PRIMARY KEY UNIQUE NOT NULL AUTO_INCREMENT,
    character_id BIGINT unsigned,
    alliance_id BIGINT unsigned,
    corporation_id BIGINT unsigned,
    faction_id BIGINT unsigned,
    damage_done BIGINT unsigned NOT NULL,
    final_blow BOOLEAN NOT NULL,
    security_status FLOAT NOT NULL,
    ship_type_id BIGINT unsigned,
    weapon_type_id BIGINT unsigned,
    killmail_id BIGINT unsigned NOT NULL,
    FOREIGN KEY (killmail_id) REFERENCES killmails (killmail_id) ON DELETE CASCADE,
    FOREIGN KEY (character_id) REFERENCES character_public_info (character_id),
    FOREIGN KEY (ship_type_id) REFERENCES esi_types (type_id),
    FOREIGN KEY (weapon_type_id) REFERENCES esi_types (type_id)
);
CREATE TABLE killmail_positions (
    position_id BIGINT unsigned PRIMARY KEY UNIQUE NOT NULL AUTO_INCREMENT,
    x DOUBLE NOT NULL,
    y DOUBLE NOT NULL,
    z DOUBLE NOT NULL,
    killmail_id BIGINT UNSIGNED NOT NULL UNIQUE,
    FOREIGN KEY (killmail_id) REFERENCES killmails (killmail_id) ON DELETE CASCADE
);