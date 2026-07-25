{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/26.05";
  };

  outputs =
    inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "dot";
          version = (fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "tielpmet-0.2.0" = "sha256-955FzzunRsvsdcgN7OpZCtzKBpZgiyiWCXGOiFZprgI=";
            };
          };

          nativeBuildInputs = with pkgs; [
            cmake
            installShellFiles
            perl
          ];

          env.SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";

          postInstall = ''
            out_dir=$(find target -type d -path '*/build/dot-*/out' | head -1)
            installShellCompletion --cmd dot \
              --bash "$out_dir/dot.bash" \
              --zsh  "$out_dir/_dot" \
              --fish "$out_dir/dot.fish" \
              --nushell "$out_dir/dot.nu"
          '';

          meta = {
            description = "Simple dotfile manager";
            mainProgram = "dot";
            platforms = pkgs.lib.platforms.unix;
          };
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cargo
            clippy
            nixd
            nixfmt
            rustc
            rustfmt
            rust-analyzer
          ];
        };
      }
    );
}
