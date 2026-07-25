{ inputs, ... }:
{
  flake.overlays.default =
    final: _:
    let
      mkVela = import ../toolchain.nix { inherit inputs; };
    in
    {
      vela-editor = mkVela final;
    };
}
