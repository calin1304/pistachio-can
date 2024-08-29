{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.nodejs

    # keep this line if you use bash
    pkgs.bashInteractive
  ];
}
