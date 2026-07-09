{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
        overlays = [ (import rust-overlay) ];
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "rust-analyzer" ];
        targets = [ "wasm32-unknown-unknown" "wasm32-unknown-emscripten" ];
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
        # X11 dependencies
          libx11
          libx11.dev
          libxcursor
          libxi
          libxinerama
          libxrandr

          libGL
          glfw3
          emscripten
          pkg-config
          cmake
          rustToolchain
          wabt
          llvmPackages.bintools
        ];

        RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        shellHook = ''
          export EM_CACHE="$PWD/.emscripten_cache"
          export EMCC_CFLAGS="-I${pkgs.emscripten}/share/emscripten/cache/sysroot/include"
          export LD_LIBRARY_PATH="${pkgs.libGL}/lib:${pkgs.glfw3}/lib:$LD_LIBRARY_PATH"
          export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
          export CPATH="${pkgs.glibc.dev}/include"
        '';
      };
    };
}
