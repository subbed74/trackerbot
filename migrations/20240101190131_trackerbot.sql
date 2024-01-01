-- Add migration script here
CREATE TABLE `server_bookmarks` (
    `id` INT PRIMARY KEY NOT NULL AUTO_INCREMENT,
    `guild_id` BIGINT UNSIGNED NOT NULL,
    `bookmark_name` TEXT NOT NULL,
    `host` TEXT NOT NULL,
    `port` INT UNSIGNED NOT NULL
);

CREATE TABLE `guild_settings` (
    `id` INT PRIMARY KEY NOT NULL AUTO_INCREMENT,
    `guild_id` BIGINT UNSIGNED NOT NULL,
    `infocmds_required_role` BIGINT UNSIGNED
)