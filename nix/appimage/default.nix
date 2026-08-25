{ inputs, ... }: {
  perSystem =
    { pkgs, system, ... }:
    let
      apprun = pkgs.pkgsStatic.runCommandCC "AppRun" { } ''
        mkdir -p $out/mountroot
        cp ${inputs.nix-appimage}/appruns/userns-chroot/main.c main.c
        patch main.c ${./merge-host-store.patch}
        $CC main.c -o $out/AppRun
      '';
    in
    {
      _module.args.mkAppImage = pkgs.callPackage "${inputs.nix-appimage}/mkAppImage.nix" {
        mkappimage-runtime =
          inputs.nix-appimage.packages.${system}.appimage-runtimes.appimage-type2-runtime;
        mkappimage-apprun = apprun;
      };
    };
}
