{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      lib,
      system,
      ...
    }:
    let
      mkVela = import ../toolchain.nix { inherit inputs; };
      vela-editor = mkVela pkgs;
    in
    {
      packages = {
        default = vela-editor;
        debug = vela-editor.override { profile = "dev"; };
      };
    }
    // lib.optionalAttrs (lib.hasSuffix "linux" system) {
      checks = {
        a11y-test = import ../tests/a11y.nix {
          inherit pkgs inputs;
        };
      }
      // import ../tests/sandboxing { inherit pkgs inputs; };
    };
}
