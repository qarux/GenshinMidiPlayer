# GenshinMidiPlayer

Play midi files on Windsong Lyre and Floral Zither.

## Usage
````
USAGE:
    genshin-midi-player <PATH>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <PATH>    A midi file to play
````
1. [Download](https://github.com/qarux/GenshinMidiPlayer/releases/latest) and run.
2. Switch back to Genshin Impact.
3. Equip and use Windsong Lyre or Floral Zither.
4. Press "\\" to play/pause.
5. Press ctrl + c in the terminal to quit.

## Building
### Dependencies

- X11 (libx11-dev)

The latest Rust stable (at least 1.56.1) is needed.

````
git clone https://github.com/qarux/genshin-midi-player.git
cd genshin-midi-player
cargo build --release
````

## Limitations
Wayland is not supported.