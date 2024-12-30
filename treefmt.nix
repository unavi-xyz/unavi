{ ... }:
{
  projectRootFile = "flake.nix";
  programs = {
    nixfmt.enable = true;
    prettier.enable = true;
    rustfmt.enable = true;
    taplo.enable = true;
    terraform.enable = true;
  };
}
