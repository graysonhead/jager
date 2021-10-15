CREATE TABLE factions (
    faction_id BIGINT unsigned PRIMARY KEY UNIQUE NOT NULL,
    corporation_id BIGINT unsigned,
    militia_corporation_id BIGINT unsigned,
    name TEXT NOT NULL
);
CREATE TABLE alliances (
    alliance_id BIGINT unsigned PRIMARY KEY UNIQUE NOT NULL,
    faction_id BIGINT unsigned,
    name TEXT NOT NULL,
    ticker TEXT NOT NULL,
    FOREIGN KEY (faction_id) REFERENCES factions (faction_id)
);
CREATE TABLE corporations (
    corporation_id BIGINT unsigned PRIMARY KEY UNIQUE NOT NULL,
    alliance_id BIGINT unsigned,
    faction_id BIGINT unsigned,
    member_count BIGINT unsigned,
    name TEXT NOT NULL,
    ticker TEXT NOT NULL,
    war_eligible BOOL,
    FOREIGN KEY (alliance_id) REFERENCES alliances (alliance_id),
    FOREIGN KEY (faction_id) REFERENCES factions (faction_id)
);
ALTER TABLE victims
ADD FOREIGN KEY (alliance_id) REFERENCES alliances (alliance_id),
    ADD FOREIGN KEY (corporation_id) REFERENCES corporations (corporation_id),
    ADD FOREIGN KEY (faction_id) REFERENCES factions (faction_id);
ALTER TABLE attackers
ADD FOREIGN KEY (alliance_id) REFERENCES alliances (alliance_id),
    ADD FOREIGN KEY (corporation_id) REFERENCES corporations (corporation_id),
    ADD FOREIGN KEY (faction_id) REFERENCES factions (faction_id);