Rusty Aquarium
==============
[![Cross-compile](https://github.com/ollej/rusty-aquarium/actions/workflows/crosscompile.yml/badge.svg)](https://github.com/ollej/rusty-aquarium/actions/workflows/crosscompile.yml)

A monitoring visualization as an aquarium written in Rust.

![Screenshot](https://ollej.github.io/rusty-aquarium/assets/screenshot.png)

Demo
----

Run the Rusty Aquarium in your browser:

 * [Web demo](https://ollej.github.io/rusty-aquarium/demo/)

Download a Windows exe or an Android APK:

 * [Latest Release](https://github.com/ollej/rusty-aquarium/releases/latest)

License
=======

The code for Rusty Aquarium is released under the MIT License.
See LICENSE file for more details.

The images are not covered by the license, and are to the best of my knowledge
public domain.

Build scripts
-------------

The build scripts require the `cargo-run-scripts` package to be installed.

```
cargo install cargo-run-script
```

### Build a web package in `demo/`
```
cargo run-script build-web
```

### Start a web server
```
cargo run-script serve-web
```

### Build windows binary
```
cargo run-script build-win
```

### Build Android APK
```
cargo run-script build-apk
```

Generate input data
-------------------

The file `assets/inputdata.json` is read to define what fish to display.

### File format

```json
{
    "school": [
        { "fish": "crab", "size": 1.0, "speed": 1.0 },
    ]
}
```

### System monitoring

The `systemdata` binary generates an inputdata.json file based on CPU,
processes and disks.

```bash
cd src/lib/systemdata; cargo run > ../../../assets/inputdata.json
```
