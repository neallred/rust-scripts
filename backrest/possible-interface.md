# Backrest

Notes:

Allows overlapping patterns of backups. By default, if there is an overlap, the more "specific" pattern will include the file/folder used. If neither is more specific, the first file/pattern specified will contain the matched files and folders. Exlusive overlap can be turned off by passing --inclusive-overlap (or by running the command multiple times)

Usage:

subcommands

version     print version info
backup      create backup archive
restore     restore

--no-security            skip encryption
--no-compression         skip compression
--include-gitignore      include gitignored files/folders 
--exclude [file/pattern] exclude from file
--inclusive-overlap      f
--destination            If not passed, defaults to $HOME/backrest. If existing folder is passed, will place inside
[file/folder/pattern,name patterns]          Comm

Examples

Open questions
Can should i embed
 age 
 tag in+ the ------------ archive file?
