{ nixpkgs }:
let
  show = nixpkgs.stdenv.mkDerivation {
    name = "show";
    buildPhase = ''
      ${nixpkgs.terraform}/bin/terraform show -json > $out
    '';
  };

  resourcesInModule = type: module:
    builtins.filter (r: r.type == type) module.resources ++ nixpkgs.lib.flatten
    (map (resourcesInModule type) (module.child_modules or [ ]));

  resourcesByType = type: resourcesInModule type show.values.root_module;
in { inherit resourcesByType; }
