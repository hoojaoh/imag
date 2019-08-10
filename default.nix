{ pkgs ? (import <nixpkgs> {}) }:

let
  env = with pkgs.rustChannels.stable; [
    rust
    cargo
  ];

  dependencies = with pkgs; [
    cmake
    curl
    gcc
    libpsl
    openssl
    pkgconfig
    which
    zlib
    dbus
    libtool
  ];
in

pkgs.stdenv.mkDerivation rec {
    name = "imag";
    src = /var/empty;
    version = "0.0.0";

    buildInputs = env ++ dependencies;

    LIBCLANG_PATH="${pkgs.llvmPackages.libclang}/lib";
}

