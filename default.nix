{
  pkgs ? import <nixpkgs> {}, 
  gis ? import (fetchTarball {
     url = https://github.com/icetan/nix-git-ignore-source/archive/v1.0.0.tar.gz;
     sha256 = "1mnpab6x0bnshpp0acddylpa3dslhzd2m1kk3n0k23jqf9ddz57k";
  }) {},
}:
with pkgs;
rustPlatform.buildRustPackage rec {
  pname = "cargo-feature";
  version = "0.5.2";

  src = gis.gitIgnoreSource ./.;

  cargoSha256 = "0nvl5smibl81b826xcsrjx8p89lcfpj7wqdsvywnj7jd3p5ag03n";

  meta = with lib; {
    description = "Allows conveniently modify features of crate";
    homepage = "https://github.com/Riey/cargo-feature";
    license = licenses.mit;
    platforms = platforms.unix;
    maintainers = with maintainers; [ riey ];
  };
}

