{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:
{
  env.LD_LIBRARY_PATH = builtins.concatStringsSep ":" [
    "${pkgs.xorg.libX11}/lib"
    "${pkgs.xorg.libXi}/lib"
    "${pkgs.libGL}/lib"
    "${pkgs.libxkbcommon}/lib"
  ];

  languages.rust = {
    enable = true;
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

}
