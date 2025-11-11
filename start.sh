if [ "$1" = "online" ]; then
  mprocs --names="proxy db, db shell" \
    "fly -c db-fly-io/fly.toml proxy 3306 -a sm-mysql-rabbit" \
    "mariadb --protocol=tcp -h localhost -P 3306 -u dev -p"
else
  ides start mariadb && sleep 1
  mprocs --names="serve back,serve front,db shell" \
    "PORT=3000 cargo run --manifest-path back/Cargo.toml " \
    "miniserve front --index index.html " \
    "mariadb -S ./.ides/mariadb/run/mysqld.sock -u root"
fi
