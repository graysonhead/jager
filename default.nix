{ release ? true }:
with import <nixpkgs> { };
let
    cargoNix = callPackage ./Cargo.nix {
        inherit release;
    };
in
cargoNix.workspaceMembers.backend.build