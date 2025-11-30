{
  config,
  pkgs,
  lib,
  ...
}: let
  cfg = config.services.docker;
  inherit (lib) types mkOption;
in {
  options.services.docker = {
    enable = lib.mkEnableOption "Enable Docker service.";

    extraArgs = mkOption {
      type = types.str;
      description = "Additional arguments to pass to Docker server.";
      default = "";
    };

    name = mkOption {
      type = types.str;
      description = "Name used to identify the Docker service.";
      default = "docker";
    };
  };

  config.serviceDefs = lib.mkIf cfg.enable {
    "${cfg.name}" = {
      pkg =
        pkgs.writeShellScriptBin config.serviceDefs.${cfg.name}.exec
        ''
          export DOCKER_HOST="unix://$XDG_RUNTIME_DIR/docker.sock"
          ${pkgs.docker}/bin/dockerd-rootless \
            ${cfg.extraArgs}
        '';
      exec = cfg.name;
      config.format = "ini";
    };
  };
}
