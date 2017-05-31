{ pkgs ? (import <nixpkgs> {}) }:

let
  env = with pkgs.rustStable; [
    rustc
    cargo
  ];

  dependencies = with pkgs; [
    bundler
    cmake
    curl
    gcc
    libpsl
    openssl
    pkgconfig
    which
    zlib
    dbus
    pkgconfig
  ];
in

pkgs.stdenv.mkDerivation rec {
    name = "imag";
    src = ./.;
    version = "0.0.0";

    buildInputs = env ++ dependencies;

}

