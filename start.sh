#!/bin/sh
set -o errexit
set -o nounset

if [ "${1-}" = "online" ]; then
  ides start phpMyAdmin && sleep 1
  if ! [ "${2-}" = "--no-browser" ]; then
    {
      xdg-open "http://[::]:8000/address/index.php" &&
        sleep 3 &&
        xdg-open "https://test-sm-website.fly.dev" &&
        sleep 3 &&
        xdg-open "https://studio-matic.github.io/test-website"
    } >/dev/null 2>&1 &
  fi
  mprocs --names="proxy db, db shell" \
    "fly -c db-fly-io/fly.toml proxy 3306 -b '[::]' -a sm-mysql-rabbit" \
    "mariadb --protocol=tcp -h localhost -P 3306 -u dev -p"
elif [ "${1-}" = "offline" ]; then
  ides start && sleep 1
  if ! [ "${2-}" = "--no-browser" ]; then
    {
      xdg-open "http://[::]:8000/socket/index.php" &&
        sleep 3 &&
        xdg-open "https://[::]:$PORT" &&
        sleep 3 &&
        xdg-open "http://[::]:8080"
    } >/dev/null 2>&1 &
  fi
  mprocs --names="serve back,serve front,db shell" \
    "cargo run --manifest-path back/Cargo.toml " \
    "miniserve front --pretty-urls --index index.html " \
    "mariadb -S ./.ides/mariadb/run/mysqld.sock -u root"
else
  echo "Usage: ${0##*/} {online|offline} [--no-browser]"
fi
