{ ... }:
{
  projectRootFile = "flake.nix";
  programs = {
    actionlint.enable = true;
    deadnix.enable = true;
    mdformat.enable = true;
    nixfmt.enable = true;
    rustfmt = {
      enable = true;
      edition = "2024";
    };
    taplo.enable = true;
    terraform.enable = true;
    yamlfmt.enable = true;
  };
}
