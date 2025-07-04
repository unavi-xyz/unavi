_: {
  projectRootFile = "flake.nix";
  programs = {
    actionlint.enable = true;
    deadnix.enable = true;
    mdformat.enable = true;
    nixfmt.enable = true;
    oxipng.enable = true;
    rustfmt = {
      enable = true;
      edition = "2024";
    };
    statix.enable = true;
    taplo.enable = true;
    terraform.enable = true;
    yamlfmt.enable = true;
  };
}
