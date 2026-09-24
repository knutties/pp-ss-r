{
  description = "SSR payment page (Rust + actix)";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs = { self, nixpkgs }:
    let
      forAll = f: nixpkgs.lib.genAttrs
        [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ]
        (system: f (import nixpkgs { inherit system; }) system);
    in {
      devShells = forAll (pkgs: system:
        let
          # chromium is not available in nixpkgs on darwin; only bundle it on Linux.
          # On darwin, e2e uses the system Chrome via CHROME_BIN (see scripts/e2e).
          isLinux = pkgs.stdenv.hostPlatform.isLinux;
          browser = if isLinux then [ pkgs.chromium ] else [ ];
        in {
          default = pkgs.mkShell {
            packages = [ pkgs.cargo pkgs.rustc pkgs.rustfmt pkgs.clippy ] ++ browser;
            CHROME_BIN =
              if isLinux
              then "${pkgs.chromium}/bin/chromium"
              else "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
          };
        });
    };
}
