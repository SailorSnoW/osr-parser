# Osu! replay parser for Rust

### **The project is currently in an early-development stage.**
#### Please **OPEN AN ISSUE** if you encounter any problems using the library.

---

A Rust library to read and parse contained data of an Osu! score replay file to easily manipulate replays in a Rust project.

This library was made according how a replay file is structured explained on the official wiki of Osu!
(https://osu.ppy.sh/wiki/en/Client/File_formats/Osr_%28file_format%29).

## Usage

```rust
use osr_parser::Replay;

fn main() {
    let replay_path = Path::from("./assets/examples/replay-test.osr");

    let replay: Replay = Replay::open(&replay_path).unwrap();
    
    let player_name: String = replay.player_name;
    let is_a_full_combo: bool = replay.is_full_combo;
    let miss_count: u16 = replay.number_misses;
    let first_frame: ReplayFrame = replay.replay_data.frames[0];
}
```
