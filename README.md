## How to test
[Install the `nix` package manager](https://nixos.org/download/) then run `nix develop . --no-pure-eval`.  
If it's your first time setting up the database, in the spawned shell run `ides start mariadb`, log into the dbms shell with `mariadb -S .ides/mariadb/run/mysqld.sock -u root` and `source ./setup.sql` and `exit` and `./start.sh offline`.  
Otherwise in the spawned shell you'll want to run just `./start.sh offline` to start the services.  

## How to deploy
[Install the `nix` package manager](https://nixos.org/download/) then run `nix develop . --no-pure-eval`.  
Make sure there is at least one assigned machine via `fly scale count 1` and that they are started via `fly machine start` for both `./db-fly-io` and `./back` in that order.  
Run `./start.sh online`.  
When done you can reduce costs removing all assigned machines via `fly scale count 0` for both `./db-fly-io` and `./back`.  
The front is hosted on https://studio-matic.github.io/test-website you can update it by doing `git subtree push --prefix front origin gh-pages`.  
