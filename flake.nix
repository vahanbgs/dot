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
          version = "0.6.1";
          src = ./.;

          cargoHash = "sha256-FWLHdHv+EJ5PYP1TTCN3G10RDDTGAnugBSQ2eYemKCs=";

          nativeBuildInputs = with pkgs; [
            installShellFiles
          ];

          postInstall = ''
            installShellCompletion --cmd dot \
              --bash target/*/build/dot-*/out/dot.bash \
              --zsh target/*/build/dot-*/out/_dot \
              --fish target/*/build/dot-*/out/dot.fish \
              --nushell target/*/build/dot-*/out/dot.nu
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
