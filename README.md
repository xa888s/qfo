# qfo (pronounced queue-foh)
This program runs in the background and switches your layers on a QMK keyboard
(right now just the Ergodox EZ) depending on which window is active (X11 on
Linux only for now).

## Uses
For example, this allows you to have a game layer that is activated whenever
your game is focused, and switch to your base layer whenever you are outside of
games. However this doesn't just apply to games, any program that you can find
the WM_CLASS for can be used.

## Install
Clone the repo somewhere,
```
git clone https://git.cryptid.cc/lost/qfo
```
First you will need to build your QMK firmware with `RAW_ENABLE` set to `yes` in
your `rules.mk`, which should be next to your keymap.

### WARNING
This may mean you need to disable some other features to make sure you have
enough space, which is possibly undesirable (Although I only had to disable
`ORYX_ENABLE` and `WEBUSB_ENABLE`).

Then you must include the contents of `snippet.c` somewhere AFTER your keymap
configuration (hopefully I can find a way to automate this in the future).

When all of the above is done and your firmware builds correctly you can then
build and run this program with the usual commands for a Rust project (compile
in release for better performance/efficiency)
```sh
cd qfo
cargo run --release
```
Now the program is ready to be configured (you might have noticed it doesn't do
anything right now :D).

## Configuration
If you have run the program before, it should have generated a default
configuration file (if not you can use the one in this repository: `config.ron`). You
can then modify this to your needs (filling the lists with your WM_CLASS names).
