{
  description = "git-tree — terminal-style git branch visualizer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Libraries Dioxus desktop (wry/tao + webkitgtk) and rfd link against
        runtimeLibs = with pkgs; [
          webkitgtk_4_1
          gtk3
          glib
          cairo
          pango
          gdk-pixbuf
          atk
          libsoup_3
          xdotool # provides libxdo (pulled in by dioxus-desktop -> muda)
          openssl
          zlib
          wayland
          libxkbcommon
        ];

        buildTools = with pkgs; [
          pkg-config
          perl # needed by the vendored openssl build
          wrapGAppsHook3 # sets GSettings schemas / XDG_DATA_DIRS so the file picker works
        ];

        git-tree = pkgs.rustPlatform.buildRustPackage {
          pname = "git-tree";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = buildTools;
          buildInputs = runtimeLibs;

          # Tests may need a display / network; skip them in the Nix build
          doCheck = false;

          # Black/blank window fix on many NixOS setups (NVIDIA, some Wayland compositors).
          # main.rs sets this too; kept here as a safety net.
          preFixup = ''
            gappsWrapperArgs+=(
              --set-default WEBKIT_DISABLE_DMABUF_RENDERER 1
            )
          '';

          postInstall = ''
            install -Dm644 assets/icon/png/icon.png \
              $out/share/icons/hicolor/256x256/apps/git-tree.png

            mkdir -p $out/share/applications
            cat > $out/share/applications/git-tree.desktop <<EOF
            [Desktop Entry]
            Name=git-tree
            Comment=Terminal-style git branch visualizer
            Exec=git-tree
            Icon=git-tree
            Terminal=false
            Type=Application
            Categories=Development;RevisionControl;
            EOF
          '';

          meta = with pkgs.lib; {
            description = "Terminal-style git branch visualizer";
            homepage = "https://github.com/MahiroJV/git-tree";
            license = licenses.mit;
            mainProgram = "git-tree";
            platforms = platforms.linux;
          };
        };
      in
      {
        packages.default = git-tree;
        packages.git-tree = git-tree;

        apps.default = flake-utils.lib.mkApp { drv = git-tree; };

        # `nix develop` → shell for working on the project
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = buildTools ++ (with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            dioxus-cli
          ]);
          buildInputs = runtimeLibs;

          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH
            export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
            export WEBKIT_DISABLE_DMABUF_RENDERER=1
          '';
        };
      });
}
