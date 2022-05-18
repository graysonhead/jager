{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/master";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs";
      flake-compat = {
        url = github:edolstra/flake-compat;
        flake = false;
    };
  };

   outputs = { self, nixpkgs, cargo2nix, flake-utils, rust-overlay, ... }:

    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import "${cargo2nix}/overlay")
                      rust-overlay.overlay];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet' {
          rustChannel = "1.60.0";
          packageFun = import ./Cargo.nix;
        };

        # workspaceShell = rustPkgs.workspaceShell {};
        # Temporary workaround, see https://github.com/cargo2nix/cargo2nix/issues/238
        workspaceShell = pkgs.mkShell {
          packages = with pkgs; [
            python3
            openssl.dev
            pkg-config
            cairo
            gobject-introspection
            atk
            gdk-pixbuf
            pango
            gtk3
            zlib
            xorg.libxcb
            cargo
            cargo-watch
            rustc
            rust-analyzer
          ];
        };

        #Output set
        in rec {
          packages = {
            jager-client = (rustPkgs.workspace.jager-client {}).bin;
            jager-backend = (rustPkgs.workspace.backend {}).bin;
          };
        overlay = final: prev: {
          jager = packages.${system};
        };
        devShell = workspaceShell;
        defaultPackage = packages.jager-client;
        nixosModule = { config, lib, pkgs, ... }:
          with lib;
            let cfg = config.jager.services.api-server;
          in {
            options.jager.services.api-server = {
              enable = mkEnableOption "Enables jager api-server service";
            };

            config = mkIf cfg.enable {
              systemd.services."jager.api-server" = {
                wantedBy = [ "multi-user.target" ];

                serviceConfig = let pkg = self.packages.${system}.jager-backend;
                in {
                  Restart = "on-failure";
                  ExecStart = "${pkg}/bin/api-server";
                };
              };
            };
          };
        }
    );
}