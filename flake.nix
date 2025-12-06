{
  description = "Rust backend for studiomatic's website";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    ides.url = "git+https://git.atagen.co/atagen/ides";
    oxalica-rust-overlay = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "github:oxalica/rust-overlay";
    };
  };

  outputs = {
    self, # necessary
    nixpkgs,
    oxalica-rust-overlay,
    ides,
  }: let
    projectDir =
      # SPECIFY PROJECT DIRECTORY HERE, use an absolute path and then comment the following lines
      let
        pwdCheck = builtins.tryEval (builtins.getEnv "PWD");
      in
        if pwdCheck.success && pwdCheck.value != ""
        then pwdCheck.value
        else throw "This flake must either be run with --no-pure-eval to resolve the current working directory as the project directory, or must be modified to specify a a project directory!";

    forAllSystems = nixpkgs.lib.genAttrs [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];
    pkgsForAllSystems = f:
      forAllSystems (system:
        f (import nixpkgs {
          overlays = [(import oxalica-rust-overlay)];
          inherit system;
        }));
  in {
    devShells = pkgsForAllSystems (pkgs: {
      default = let
        dataDir = "${projectDir}/.ides/mariadb";
        socket = "${dataDir}/run/mysqld.sock";
      in
        (import ides {
          inherit pkgs;
          shell = pkgs.mkShell.override {
            stdenv = pkgs.stdenvNoCC;
          };
          auto = false;
        })
        {
          packages = with pkgs;
            [
              taplo
              (rust-bin.fromRustupToolchainFile ./back/rust-toolchain.toml)
              vscode-langservers-extracted
              typescript-language-server
            ]
            ++ [
              mprocs
              mariadb-client
              docker
              miniserve
              flyctl
              xdg-utils
              sqlx-cli
            ];
          imports = [
            ./nix/mysql.nix
            ./nix/docker.nix
            ./nix/phpMyAdmin
          ];
          services = {
            mysql.enable = true;
            mysql.dataDir = dataDir;
            docker.enable = true;
            phpMyAdmin.enable = true;
          };
          PORT = 3000;
          DATABASE_URL = "mysql://root@localhost/db?socket=${socket}";
          shellHook = ''
            export DOCKER_HOST="unix://$XDG_RUNTIME_DIR/docker.sock"
          '';
        };
    });
  };
}
