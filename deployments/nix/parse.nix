{ pkgs ? import <nixpkgs> { }, lib ? pkgs.lib }:
let
  resourcesInModule = type: module:
    builtins.filter (r: r.type == type) module.resources
    ++ lib.flatten (map (resourcesInModule type) (module.child_modules or [ ]));
  resourcesByType = type: resourcesInModule type payload.values.root_module;
  payload = builtins.fromJSON (builtins.readFile ./show.json);
in { inherit resourcesByType; }
