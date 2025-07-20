{ pkgs, lib, config, inputs, ... }:

{
  packages = with pkgs; [
    bacon
  ];

  languages.rust.enable = true;
  languages.rust.channel = "stable";
  languages.rust.mold.enable = true;
}
