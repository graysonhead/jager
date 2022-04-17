# SPDX-FileCopyrightText: 2021 Serokell <https://serokell.io/>
#
# SPDX-License-Identifier: CC0-1.0

{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crate2nix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        crateName = "jager";

        inherit (import "${crate2nix}/tools.nix" { inherit pkgs; })
          generatedCargoNix;

        project = import (generatedCargoNix {
          name = crateName;
          src = ./.;
        }) {
          inherit pkgs;
          defaultCrateOverrides = pkgs.defaultCrateOverrides // {
              cairo-sys-rs = _: {
                  nativeBuildInputs = [ pkgs.pkg-config pkgs.cairo ];
              };
              gobject-sys = _: {
                  nativeBuildInputs = [ pkgs.pkg-config pkgs.gobject-introspection ];
              };
              atk-sys = _: {
                  nativeBuildInputs = [ pkgs.pkg-config pkgs.atk ];
              };
              gio-sys = _: {
                  nativeBuildInputs = [ pkgs.pkg-config pkgs.haskellPackages.gi-gio ];
              };
              gdk-pixbuf-sys = _: {
                  nativeBuildInputs = [ pkgs.pkg-config pkgs.gdk-pixbuf ];
              };
              pango-sys = _: {
                  nativeBuildInputs = [ pkgs.pkg-config pkgs.pango ];
              };
              gdk-sys = _: {
                  nativeBuildInputs = [ pkgs.pkg-config pkgs.gtk3 ];
              };
              gtk-sys = _: {
                  nativeBuildInputs = [ pkgs.pkg-config pkgs.gtk3 ];
              };
              jager-client = _: {
                  buildInputs = [ pkgs.zlib pkgs.xorg.libxcb ];
              };
          };
        };

      in {
        packages.${crateName} = project.workspaceMembers.backend.build;
        packages.jager-client = project.workspaceMembers.jager-client.build;

        defaultPackage = self.packages.${system}.jager-client;

        devShell = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.packages.${system};
          buildInputs = [ pkgs.cargo pkgs.rust-analyzer pkgs.clippy ];
        };
      });
}
