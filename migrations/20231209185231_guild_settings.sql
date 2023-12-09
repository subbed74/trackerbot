-- Add migration script here
CREATE TABLE guild_settings(
    id INTEGER PRIMARY KEY NOT NULL,
    guild_id INTEGER NOT NULL,
    infocmds_required_role INTEGER
)