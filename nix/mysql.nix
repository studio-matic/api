{
  config,
  pkgs,
  lib,
  ...
}: let
  cfg = config.services.mysql;
  inherit (lib) types mkOption;
in {
  options.services.mysql = {
    enable = lib.mkEnableOption "Enable MariaDB service.";

    dataDir = mkOption {
      type = types.str;
      description = "Data directory for MariaDB. Provide an absolute path as a string.";
    };

    socket = mkOption {
      type = types.str;
      description = "Unix socket file path for MariaDB.";
      default = "${cfg.dataDir}/run/mysqld.sock";
    };

    # bindAddress = mkOption {
    #   type = types.str;
    #   description = "Address MariaDB will bind to.";
    #   default = "[::]";
    # };

    # port = mkOption {
    #   type = types.int;
    #   description = "Port MariaDB will bind to.";
    #   default = 3306;
    # };

    extraArgs = mkOption {
      type = types.str;
      description = "Additional arguments to pass to MariaDB server.";
      default = "";
    };

    name = mkOption {
      type = types.str;
      description = "Name used to identify the MariaDB service.";
      default = "mariadb";
    };
  };

  config.serviceDefs = lib.mkIf cfg.enable {
    "${cfg.name}" = {
      pkg =
        pkgs.writeShellScriptBin config.serviceDefs.${cfg.name}.exec
        ''
            mkdir -p "${cfg.dataDir}"
            mkdir -p "$(dirname "${cfg.socket}")"
            if [ ! -f "${cfg.dataDir}/mysql/user.frm" ]; then
              echo "Initializing MariaDB data directory at ${cfg.dataDir}"
              ${pkgs.mariadb}/bin/mariadb-install-db \
                --no-defaults \
                --datadir="${cfg.dataDir}" \
                --auth-root-authentication-method=normal
            fi
          ${pkgs.mariadb}/bin/mariadbd \
            --no-defaults \
            --datadir="${cfg.dataDir}" \
            --skip-networking \
            --socket="${cfg.socket}" \
            ${cfg.extraArgs}
        '';
      exec = cfg.name;
      config.format = "ini";
    };
  };
}
