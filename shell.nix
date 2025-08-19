{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
    nativeBuildInputs = with pkgs; [
    ];
    
    buildInputs = with pkgs; [
        cargo
        rustPackages.clippy
        rust-analyzer
        rustc
        rustfmt
    ];
    
    shellHook = ''
        export TMPDIR=/tmp/$USER-rust
        mkdir -p "$TMPDIR"
    '';
}
