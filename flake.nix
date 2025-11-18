{
  description = "aim";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
  };

  outputs =
    {
      self,
      nixpkgs,
      systems,
      ...
    }:

    let
      inherit (nixpkgs) lib;

      transposeAttrs =
        attrs:
        let
          keys = lib.attrNames attrs;
          subkeys = lib.attrNames (lib.head (lib.attrValues attrs));
        in
        lib.genAttrs subkeys (subkey: lib.genAttrs keys (key: attrs.${key}.${subkey}));

      forEachSupportedSystem = f: transposeAttrs (lib.genAttrs (import systems) f);

      aim-package =
        {
          lib,
          stdenv,
          rustPlatform,
          sqlite,
          installShellFiles,
          testers,
        }:

        rustPlatform.buildRustPackage (finalAttrs: {
          pname = "aim";
          version = self.shortRev or self.dirtyShortRev or "unknown";

          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./aimcal
              ./cli
              ./core
              ./ical
              ./Cargo.toml
              ./Cargo.lock
            ];
          };

          cargoLock = {
            # NOTE: This is only used for Git dependencies
            allowBuiltinFetchGit = true;
            lockFile = ./Cargo.lock;
          };

          buildNoDefaultFeatures = true;
          buildFeatures = [ "sqlite-unbundled" ];

          nativeBuildInputs = [
            rustPlatform.bindgenHook
            installShellFiles
          ];

          buildInputs = [
            sqlite
          ];

          postInstall = lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
            installShellCompletion --cmd aim \
              --bash <($out/bin/aim generate-completion bash) \
              --fish <($out/bin/aim generate-completion fish) \
              --zsh <($out/bin/aim generate-completion zsh)
          '';

          passthru.tests = {
            version = testers.version {
              package = finalAttrs.finalPackage;
              version = finalAttrs.version;
            };
          };

          meta = {
            description = "Analyze. Interact. Manage Your Time, with calendar support";
            homepage = "https://github.com/yzx9/aim";
            license = lib.licenses.asl20;
            platforms = lib.platforms.all;
            maintainers = with lib.maintainers; [ yzx9 ];
            mainProgram = "aim";
          };
        });
    in
    forEachSupportedSystem (
      system:

      let
        pkgs = nixpkgs.legacyPackages.${system};
        aim = pkgs.callPackage aim-package { };
      in
      {
        devShells.default = pkgs.mkShell {
          env = {
            RUST_LOG = "info";
          };

          packages = with pkgs; [
            # rust toolchain
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer

            # cargo tools
            cargo-audit
            cargo-release
            cargo-outdated

            # misc
            just
            sqlx-cli
          ];
        };

        packages = {
          default = aim;
          inherit aim;
        };
      }
    )
    // {
      overlays.default = final: _: {
        aim = final.callPackage aim-package { };
      };
    };
}
