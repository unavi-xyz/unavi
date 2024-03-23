{ pkgs }:
let
  show = pkgs.stdenv.mkDerivation {
    name = "show";
    buildInputs = with pkgs; [ jq terraform ];

    src = ./terraform;

    buildPhase = ''
      export AWS_ACCESS_KEY_ID=${builtins.getEnv "AWS_ACCESS_KEY_ID"}
      export AWS_SECRET_ACCESS_KEY=${builtins.getEnv "AWS_SECRET_ACCESS_KEY"}
      export DIGITALOCEAN_TOKEN=${builtins.getEnv "DIGITALOCEAN_TOKEN"}

      terraform init
      terraform show -json | jq "[.values.root_module.resources[] | {name,type,values: .values | {name,ipv4_address}}]" > show.json
    '';
    installPhase = ''
      mkdir -p $out
      cp show.json $out/show.json
    '';
  };

  payload = builtins.fromJSON (builtins.readFile "${show}/show.json");

  resourcesInModule = type: module: builtins.filter (r: r.type == type) module;

  resourcesByType = type: resourcesInModule type payload;
in { inherit resourcesByType; }
