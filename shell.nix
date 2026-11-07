# Classic (non-flake) Nix shell for building and running git-tree from source.
#
#   nix-shell
#   cargo run --release
#
# Building from source links against nixpkgs' own libxdo (libxdo.so.4), so the
# prebuilt Ubuntu binary's libxdo.so.3 problem doesn't apply.
{ pkgs ? import <nixpkgs> { } }:

let
  runtimeLibs = with pkgs; [
    webkitgtk_4_1
    gtk3
    glib
    cairo
    pango
    atk
    gdk-pixbuf
    libsoup_3
    xdotool # provides libxdo (pulled in by dioxus-desktop -> muda)
    openssl
    zlib
    wayland
    libxkbcommon
  ];
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    perl # vendored openssl build
    cargo
    rustc
    rustfmt
    clippy
    dioxus-cli
  ];

  buildInputs = runtimeLibs;

  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH

    # GSettings schemas so the GTK file picker (rfd) doesn't crash
    export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS

    # Black-window workaround (also set in main.rs, kept here for older builds)
    export WEBKIT_DISABLE_DMABUF_RENDERER=1

    echo "git-tree dev shell — run: cargo run --release"
  '';
}
