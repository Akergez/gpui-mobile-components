#!/usr/bin/env bash
# Screenshots of the gallery in both skins, under a private headless sway.
# Run inside the UI image (sway + grim), from the repository root:
#
#   build-aux/cargo.sh build -p gallery
#   podman run --rm --userns=keep-id --cap-add=SYS_NICE --security-opt label=disable \
#     -v "$PWD":/src -v "$HOME/.cargo/registry":"$HOME/.cargo/registry":ro -w /src \
#     localhost/aiddiction-ui build-aux/shots.sh
#
# Each line below is: name, size, input, route, demo. Pictures go to
# target/shots/<name>.png.
set -Eeuo pipefail
shots=target/shots
mkdir -p "$shots"
runtime=$(mktemp -d)
chmod 700 "$runtime"
export XDG_RUNTIME_DIR=$runtime
unset DISPLAY WAYLAND_DISPLAY
printf '%s\n' 'default_border none' 'default_floating_border none' > "$runtime/sway.conf"
WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 WLR_RENDERER=pixman \
  sway --unsupported-gpu -c "$runtime/sway.conf" >"$runtime/sway.log" 2>&1 &
compositor=$!
trap 'kill $compositor 2>/dev/null || true; rm -rf "$runtime"' EXIT
for _ in $(seq 100); do
  socket=$(find "$runtime" -maxdepth 1 -name 'wayland-*' ! -name '*.lock' -print -quit)
  [[ -n $socket ]] && break
  sleep 0.1
done
export WAYLAND_DISPLAY=${socket##*/}
SWAYSOCK=$(find "$runtime" -maxdepth 1 -name 'sway-ipc.*' -print -quit)
export SWAYSOCK

while read -r name size input route demo; do
  [[ -z $name || $name == \#* ]] && continue
  swaymsg -q output HEADLESS-1 resolution "$size"
  style=()
  [[ $name == aiddiction-* ]] && style=(GALLERY_AIDDICTION=1)
  env "${style[@]}" GALLERY_SIZE=$size GALLERY_INPUT=$input GALLERY_ROUTE=$route GALLERY_DEMO=${demo#-} \
    target/debug/gallery >"$runtime/$name.log" 2>&1 &
  app=$!
  sleep "${SHOT_WAIT:-3}"
  grim "$shots/$name.png" || true
  kill $app 2>/dev/null || true
  wait $app 2>/dev/null || true
  echo "shot  $name"
done <<'LIST'
desktop-lists      1280x800 pointer lists    -
desktop-person     1280x800 pointer person   -
desktop-input      1280x800 pointer input    -
desktop-form       1280x800 pointer input    form
desktop-records    1280x800 pointer records  -
desktop-feedback   1280x800 pointer feedback notice
desktop-confirm    1280x800 pointer feedback confirm
medium-lists        900x700 pointer lists    -
phone-lists         400x800 touch   lists    -
phone-person        400x800 touch   person   -
phone-input         400x800 touch   input    -
phone-form          400x800 touch   input    form
phone-records       400x800 touch   records  -
phone-feedback      400x800 touch   feedback notice
phone-actions       400x800 touch   lists    actions
phone-lookup        400x800 touch   lists    lookup
tablet-lists        900x700 touch   person   -
desktop-pick       1280x800 pointer lists    -
phone-error         400x800 touch   feedback error
desktop-error      1280x800 pointer feedback error
aiddiction-lists   1280x800 pointer lists    -
aiddiction-person  1280x800 pointer person   -
LIST
