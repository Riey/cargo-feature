{
  pkgs ? import <nixpkgs> {}, 
}:
with pkgs;
rustPlatform.buildRustPackage rec {
  pname = "cargo-feature";
  version = "0.6.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  meta = with lib; {
    description = "Cargo plugin to manage dependency features";
    homepage = "https://github.com/Riey/cargo-feature";
    license = licenses.mit;
    platforms = platforms.unix;
    maintainers = with maintainers; [ riey ];
  };
}

