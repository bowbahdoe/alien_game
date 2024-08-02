{
  description = "Development of alien_game";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      with pkgs; rec
      {
        devShell = mkShell rec {
          buildInputs = [
            openssl
            pkg-config
            libudev-zero
            alsa-lib
            fontconfig

            libxkbcommon
            libGL
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
          ];
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          WINIT_UNIX_BACKEND="x11";
          RUST_BACKTRACE="1";
        };
      }
    );
}
