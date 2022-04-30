{
    description = "erars";

    inputs = {
        nixpkgs.url = github:NixOS/nixpkgs;
        flake-utils.url = github:numtide/flake-utils;
    };

    outputs = { self, nixpkgs, flake-utils }:
        flake-utils.lib.eachDefaultSystem
            (system:
                let
                    pkgs = nixpkgs.legacyPackages.${system};
                in
                {
                    devShell = pkgs.mkShell {
                        name = "cargo-feature-shell";
                        nativeBuildInputs = with pkgs; [
                            pkg-config
                            rustfmt
                            rustc
                            cargo
                        ];
                        RUST_BACKTRACE=1;
                    };
                    defaultPackage = pkgs.rustPlatform.buildRustPackage rec {
                        pname = "cargo-feature";
                        version = "0.7.0";

                        src = ./.;

                        cargoLock = {
                            lockFile = ./Cargo.lock;
                        };

                        meta = with pkgs.lib; {
                            description = "Cargo plugin to manage dependency features";
                            homepage = "https://github.com/Riey/cargo-feature";
                            license = licenses.mit;
                            platforms = platforms.unix;
                            maintainers = with maintainers; [ riey ];
                        };
                    };
                }
            );
}
