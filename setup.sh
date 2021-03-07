#! /usr/bin/env sh

rm database.db
sqlite3 database.db ".read ./db/init.sql"
