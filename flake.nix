{
  description = "aim";

  inputs.nixpkgs.url = "nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs, ... }:

    let
      inherit (nixpkgs) lib;

      transposeAttrs =
        attrs:
        let
          keys = lib.attrNames attrs;
          subkeys = lib.attrNames (lib.head (lib.attrValues attrs));
        in
        lib.genAttrs subkeys (subkey: lib.genAttrs keys (key: attrs.${key}.${subkey}));

      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem = f: transposeAttrs (lib.genAttrs supportedSystems f);

      aim-package =
        {
          lib,
          stdenv,
          rustPlatform,
          installShellFiles,
          testers,
        }:

        rustPlatform.buildRustPackage (finalAttrs: {
          pname = "aim";
          version = self.shortRev or self.dirtyShortRev or "unknown";

          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./cli
              ./core
              ./Cargo.toml
              ./Cargo.lock
            ];
          };

          cargoLock = {
            # NOTE: This is only used for Git dependencies
            allowBuiltinFetchGit = true;
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ installShellFiles ];

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
            description = " Analyze. Interact. Manage Your Time";
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
          packages = with pkgs; [
            # rust
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer

            # misc
            just
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
