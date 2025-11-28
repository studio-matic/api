{
  stdenv,
  fetchurl,
  extraConfig,
}:
stdenv.mkDerivation rec {
  name = "phpmyadmin-${version}";
  version = "5.2.3";

  src = fetchurl {
    url = "https://files.phpmyadmin.net/phpMyAdmin/${version}/phpMyAdmin-${version}-all-languages.tar.gz";
    sha256 = "ErocQl+kBxq71OdmjJ696sCwdVpGem1tUCYSK7R8ECs=";
  };

  phases = [
    "unpackPhase"
    "installPhase"
  ];

  installPhase = ''
    mkdir -p $out
    cp -r * $out/

    cat > $out/config.inc.php <<'EOF'
      <?php
      ${extraConfig}
      ?>
      'EOF'
  '';

  meta = {
    homepage = "https://www.phpmyadmin.net/";
    description = "phpMyAdmin is a free and open source administration tool for MySQL and MariaDB";
  };
}
