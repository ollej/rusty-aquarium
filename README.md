Rusty Aquarium
==============

A monitoring visualization as an aquarium written in Rust.

Demo
----

Run the Rusty Aquarium in your browser:

 * [Web demo](https://ollej.github.io/rusty-aquarium/demo/)

Download a Windows exe or an Android APK:

 * [Latest Release](https://github.com/ollej/rusty-aquarium/releases/latest)

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

