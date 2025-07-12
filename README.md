# emenu

A simple gui menu like rofi/dmenu made with [egui](https://github.com/emilk/egui) and [nucleo](https://github.com/helix-editor/nucleo).

## Installation

You can install it with cargo
```sh
cargo install --git https://github.com/Tyarel8/emenu
```

## Usage

Pipe a `\n` separated list to the app and it will open the fuzzy finder, which will
return the selected option(s) when you choose.

```sh
"one\ntwo\nthree" | emenu | cat
```

![image](https://github.com/user-attachments/assets/0e61da19-7e75-405a-a0cb-462dfd9752fa)
