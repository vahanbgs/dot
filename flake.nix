{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/25.11";
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
          version = "0.5.2";
          src = ./.;

          cargoHash = "sha256-Yo2HL3qJ+bfAk1TpCF+Jgh8ch07OQ5CtHUNiXDW+lNQ=";

          nativeBuildInputs = with pkgs; [
            installShellFiles
          ];

          postInstall = ''
            # Generate and install completions
            installShellCompletion --cmd dot \
              --bash <($out/bin/dot completions bash) \
              --fish <($out/bin/dot completions fish) \
              --zsh <($out/bin/dot completions zsh) \
          '';
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
