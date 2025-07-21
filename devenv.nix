{ pkgs, lib, config, inputs, ... }:

{
  packages = with pkgs; [
    bacon
    cargo-audit
    cargo-edit
    cargo-nextest
    cargo-shear
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    mold.enable = true;
  };

  git-hooks.hooks = {
    clippy.enable = true;
    commitizen.enable = true;
    rustfmt.enable = true;
    trufflehog.enable = true;
  };
}
