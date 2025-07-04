_: {
  projectRootFile = "flake.nix";
  programs = {
    actionlint.enable = true;
    deadnix.enable = true;
    mdformat.enable = true;
    nixfmt = {
      enable = true;
      strict = true;
    };
    oxipng.enable = true;
    rustfmt.enable = true;
    statix.enable = true;
    taplo.enable = true;
    terraform.enable = true;
    yamlfmt.enable = true;
  };
}
