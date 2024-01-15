# TrackerBot

A WIP Discord bot to show Cube2: Sauerbraten statistics using Sauertracker's API. All commands have a *user* cooldown of 10 seconds. There is a 3 minute timeout present on requests to Sauertracker currently if Sauertracker is taking longer to respond.

## Commands
- () = Required parameter
- [] = Optional parameter

### Server Commands
- `/listservers` - Lists up to 10 active servers excluding Pre-2020 Edition servers.
- `/server (host) [port] [player]` - Shows game information for a specific server. `[port]` will default to 28785 if left blank. Specifying a player's username will pull that player's stats from the current match.

### Player Commands
- `/findplayer (username) [country code]` - Shows a paginated list of up to 200 players with similar usernames.
- `/player (username)` - Shows historical data for a specific player.

### Clan Commands
- `/claninfo (clantag)` - Shows information for a specific clan. NOTE: You must specify the exact clantag to get that clan's info.

### Bookmark Commands
- `/bk (bookmark name)` - Shows server match information for a bookmarked server.
- `/bkadd (bookmark name) (host) [port]` - Creates a server bookmark with the given host. `[port]` will default to 28785 if left blank.
- `/bkdelete (bookmark name)` - Deletes a server bookmark.
- `/bklist` - Lists server bookmarks.

### Bot Administration
- `/setrole [discord role]` - Sets or unsets a required role to run the bot commands. Leave blank to remove this requirement for users.