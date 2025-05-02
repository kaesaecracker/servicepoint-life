# servicepoint-life

This is a small terminal app that generates conways game of life rules and simulates them on the servicepoint display.

It uses the [servicepoint](https://git.berlin.ccc.de/servicepoint/servicepoint/) library for the display commands.

## Running

With Nix flakes, you can just `nix run git+https://git.berlin.ccc.de/vinzenz/servicepoint-life`.

Otherwise, you can `cargo install --git https://git.berlin.ccc.de/vinzenz/servicepoint-life` and then run `servicepoint-life`.

If you want to poke at the code you can always check out the repo and `cargo run` it.
