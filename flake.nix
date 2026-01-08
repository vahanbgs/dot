{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    {
      self,
      flake-utils,
      naersk,
      nixpkgs,
    }:

    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk { };
      in
      {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          pname = "dot";
          version = "0.3.0";
          src = ./.;

          nativeBuildInputs = [ pkgs.installShellFiles ];

          postInstall = ''
            # Generate and install completions
            installShellCompletion --cmd dot \
              --bash <($out/bin/dot completions bash) \
              --fish <($out/bin/dot completions fish) \
              --zsh <($out/bin/dot completions zsh) \
          '';
        };

        # For `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
          ];
        };
      }
    );
}
