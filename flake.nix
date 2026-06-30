{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
        overlays = [
        ];
      };

    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [ rustfmt wabt cargo rustc llvmPackages.bintools ];
      };
    };
}
