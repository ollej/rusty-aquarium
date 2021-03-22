Rusty Aquarium
==============

A monitoring visualization as an aquarium written in Rust.

Demo
----

Run the Rusty Aquarium in your browser:

 * [Web demo](https://ollej.github.io/rusty-aquarium/demo/)

Download a windows exe file:

 * [Windows Exe](https://github.com/ollej/rusty-aquarium/releases/download/v0.1.0/rusty-aquarium-win.zip)

Download an Android APK package:

 * [Android APK](https://github.com/ollej/rusty-aquarium/releases/download/v0.1.0/rusty-aquarium.apk)

Build scripts
-------------

The build scripts require the `cargo-run-scripts` package to be installed.

```
cargo install cargo-run-script
```

### Build a web package in `public/`
```
cargo run-script build-web
```

### Start a web server
```
cargo run-script serve-web
```

### Build windows binary and copy to `public/`
```
cargo run-script build-win
```

### Build Android APK and copy to `public/`
```
cargo run-script build-apk
```

