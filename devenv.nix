{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  packages = [ pkgs.cargo ];

  languages.rust.enable = true;
}
