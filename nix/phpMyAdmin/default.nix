{
  config,
  pkgs,
  lib,
  ...
}: let
  cfg = config.services.phpMyAdmin;
  inherit (lib) types mkOption;
  mkPmaPackage = extraConfig:
    pkgs.callPackage ./package.nix {
      extraConfig =
        ''
          $i = 0;
          $i++;
          $cfg['Servers'][$i]['AllowNoPassword'] = true;
        ''
        + extraConfig;
    };
  addressPmaPackage =
    mkPmaPackage
    ''
      $cfg['Servers'][$i]['host'] = '${cfg.address}';
      $cfg['Servers'][$i]['port'] = '${toString cfg.port}';
    '';
  socketPmaPackage =
    mkPmaPackage
    (lib.optionalString (config.services.mysql.socket != null) ''
      $cfg['Servers'][$i]['socket'] = '${config.services.mysql.socket}';
    '');
in {
  options.services.phpMyAdmin = {
    enable = lib.mkEnableOption "Enable PHP service.";

    phpAddress = mkOption {
      type = types.str;
      description = "Address PHP built-in server will listen on.";
      default = "[::]";
    };

    phpPort = mkOption {
      type = types.int;
      description = "Port PHP built-in server will listen on.";
      default = 8000;
    };

    address = mkOption {
      type = types.str;
      description = "Address PHP built-in server will listen on.";
      default = "[::]";
    };

    port = mkOption {
      type = types.int;
      description = "Port PHP built-in server will listen on.";
      default = 3306;
    };

    documentRoot = mkOption {
      type = types.str;
      description = "Document root for PHP service. Provide an absolute path as a string.";
      default = toString (pkgs.symlinkJoin {
        name = "phpmyadmin-docroot";
        paths = [
          (pkgs.runCommand "phpmyadmin-address" {} ''
            mkdir -p $out
            ln -s ${addressPmaPackage} $out/address
          '')
          (pkgs.runCommand "phpmyadmin-socket" {} ''
            mkdir -p $out
            ln -s ${socketPmaPackage} $out/socket
          '')
        ];
      });
    };

    # extraConfig = mkOption {
    #   type = types.str;
    #   description = "Additional PHP configuration (ini format).";
    #   default = "";
    # };

    extraArgs = mkOption {
      type = types.str;
      description = "Additional arguments to pass to PHP.";
      default = "";
    };

    name = mkOption {
      type = types.str;
      description = "Name ides uses for this PHP service.";
      default = "phpMyAdmin";
    };
  };

  config.serviceDefs = lib.mkIf cfg.enable {
    "${cfg.name}" = {
      pkg =
        pkgs.writeShellScriptBin config.serviceDefs.${cfg.name}.exec
        "${lib.getExe pkgs.php} -S ${cfg.phpAddress}:${toString cfg.phpPort} -t ${cfg.documentRoot} ${cfg.extraArgs}";
      exec = cfg.name;
      config.format = "ini";
    };
  };
}
