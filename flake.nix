{
  description = "arrabbiata-tui - pomodoro timer TUI";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in {
      lib.withConfig = { system, apiUrl, userId, fallbackUserId ? userId }:
        let
          pkgs = pkgsFor system;
          unwrapped = self.packages.${system}.arrabbiata-tui;
        in pkgs.symlinkJoin {
          name = "arrabbiata-tui";
          paths = [ unwrapped ];
          buildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram $out/bin/arrabbiata-tui \
              --set ARRABBIATA_API_URL "${apiUrl}" \
              --set ARRABBIATA_USER_ID "${userId}" \
              --set ARRABBIATA_FALLBACK_USER_ID "${fallbackUserId}"
          '';
        };

      packages = forAllSystems (system:
        let pkgs = pkgsFor system;
        in {
          arrabbiata-tui = pkgs.rustPlatform.buildRustPackage {
            pname = "arrabbiata-tui";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.dbus ];
          };

          default = self.packages.${system}.arrabbiata-tui;
        });

      devShells = forAllSystems (system:
        let pkgs = pkgsFor system;
        in {
          default = pkgs.mkShell {
            name = "arrabbiata-dev";
            buildInputs = with pkgs; [
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer
              pkg-config
              dbus
            ];
          };
        });
    };
}
