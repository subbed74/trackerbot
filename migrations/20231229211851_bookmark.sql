-- Add migration script here
CREATE TABLE server_bookmarks (
    id INTEGER PRIMARY KEY NOT NULL,
    guild_id INTEGER NOT NULL,
    bookmark_name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 28785
)