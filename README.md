Rusty Aquarium
==============
[![Cross-compile](https://github.com/ollej/rusty-aquarium/actions/workflows/crosscompile.yml/badge.svg)](https://github.com/ollej/rusty-aquarium/actions/workflows/crosscompile.yml)

A monitoring visualization as an aquarium written in Rust.

![Screenshot](https://ollej.github.io/rusty-aquarium/assets/screenshot.png)

Demo
----

Run Rusty Aquarium in your browser:

 * [Web demo](https://ollej.github.io/rusty-aquarium/demo/)

Download executables for different platforms:

 * [Latest Release](https://github.com/ollej/rusty-aquarium/releases/latest)

Usage
-----

Use these keys to control the fish tank.

* **Escape** - Exit the program. Not available on web.
* **Left** - Toggle shader.
* **Right** - Switch to next scene.
* **Space** - Lock/Unlock automatic scene switching.
* **Enter** - Refreshes the list of fishes.
* **Up** - Add a random fish to the tank.
* **Down** - Remove the latest added fish from the tank.
* **C** - Reload `config.json` file. Does not currently reload images.
* **D** - Force reload of data file of current scene.
* **L** / **I** - Show screen with description of fishes.
* **H** - Display help screen with shortcuts.

Generate input data
-------------------

The file `assets/inputdata.json` is read to define what fish to display.

The field `school` must be an array of objects, one for each fish to
display.

The `fish` field must match a name in the map of fishes in the
`config.json` file.

The fields `size`, `speed`, and `bubbles` are multipliers that will be applied
to the corresponding value from the fish configuration.

The `legend` field is optional, and can be used to send information to show
when opening the legend screen. The `description` will be shown at the top,
and each fish_legend will be shown in a list below it. If present, this will
override the legend field in the scene configuration.

### Example

```json
{
    "legend": {
        "description": "A description to be shown when showing legend.",
        "fish_legends": [
            {
                "fish": "crab",
                "description": "A description of this fish type."
            }
        ]
    },
    "school": [
        { "fish": "crab", "size": 1.0, "speed": 1.0, "bubbles": 1.0 },
    ]
}
```

### System monitoring

The `systemdata` binary generates an inputdata.json file based on CPU,
processes and disks.

```bash
cargo run --features build-systemdata --bin systemdata > assets/inputdata.json
```

### Google Sheets Data

The `googlesheetsdata` binary generates an inputdata.json file from a Google
Sheets spreadsheet. A file called `credentials.json` is needed that contains
Google API credientials. Use the flag `--interval N` to keep the program
running and recreate the `inputdata.json` file every N seconds. Use the flag
`--help` for more options.

The sheet needs to have a row for every fish to show. The columns need to be
`fishname`, `size`, `speed`, and `bubbles`. The `fishname` need to match the
name of a configured fish, and the other values need to be a number.

Information on how to create a `credentials.json` file can be found here:
https://developers.google.com/workspace/guides/configure-oauth-consent

```bash
cargo run --features build-googlesheetsdata --bin googlesheetsdata -- --spreadsheet SPREADSHEET-ID --output assets/inputdata.json
```

#### Usage

These are the available options:

```
A small tool to read data from Google Sheets and export to Rusty Aquarium

Usage: googlesheetsdata [OPTIONS] --spreadsheet <SPREADSHEET>

Options:
  -s, --spreadsheet <SPREADSHEET>  Spreadsheet ID to read
  -o, --output <OUTPUT>            Path to output file to store json data [default: inputdata.json]
  -i, --interval <INTERVAL>        Automatically regenerate the JSON file every N seconds
  -r, --range <RANGE>              Range of values to get from spreadsheet, like the name of a sheet [default: Sheet1]
  -c, --credentials <CREDENTIALS>  Path to Google OAuth2 credentials json file [default: credentials.json]
  -t, --tokencache <TOKENCACHE>    Path to file to store token authentication cache [default: tokencache.json]
  -h, --help                       Print help
```

### CSV Data

The `csvdata` binary generates an inputdata.json file from a comma separated
file of data. Use the flag `--listen` to keep the program running and update
the `inputdata.json` file every time the CSV file is changed. Use the flag
`--help` for more options.

The columns in the CSV files are `fishname`, `count`, and `description`. The
`count` defines how many fishes of the type `fishname` to create. The
`description` field is not used.

```bash
cargo run --features build-csvdata --bin csvdata -- --file csvfile.csv --output assets/inputdata.json
```

#### Usage

These are the available options:

```
Usage: csvdata [OPTIONS]

Options:
  -f, --file <FILE>      Path to input CSV file to convert [default: fishdata.csv]
  -o, --output <OUTPUT>  Path to output file to store json data [default: inputdata.json]
  -l, --listen           Listen to changes in file and automatically update output file
  -h, --help             Print help
```

Configuration
-------------

The file `assets/config.json` can be used to configure the fish tank. It has
some general configuration values, paths to PNG files to load for background
images and fish sprites. It also defines which fish types are available,
with default values for them.

### File format

 * **input_data_path** - Path to local file or full URL to inputdata.json file.
 * **display_time** - Number of seconds between data reloads (0 to never reload)
 * **backgrounds** - An array of strings with filenames of PNG images to use as
 background images, loaded from the `assets` directory.
 * **water_sprite** - Path to PNG image of water bubbles.
 * **scenes** - An array of scene configurations. If no scenes are configured,
   the top level values for `input_data_path` and `display_time` will be used
   instead
   * **input_data_path** - Path to load when showing this scene
   * **display_time** - Number of seconds to show scene before switching to next
   * **background** - The number of the background texture to show for this scene
   * **legend** - (Optional) Legend information to display on this scene (same
   format as in inputdata)
 * **fishes** - A list of fish type definitions, the key is used to select the
 fish type in `inputdata.json`
    * **texture** - Path to the PNG file to use for this fish.
    * **size** - Max size to scale the image to.
    * **size_randomness** - A multiplier used when randomizing fishes. Should
    be between 0.0 and 1.0.
    * **movement** - Name of the type of movement for this fish. Available
    movements: SingleSpeed, Accelerating, AcceleratingEdgeIdling, Crab, Random,
    * **bubbles** - Number of movement bubbles to show after this fish. Set to
    `0` to not display any bubbles.
    * **collision_aversion** - A number between 0 and 1. The higher the
    number, the less chance that the fish changes direction when colliding.
    * **speed** - The speed of the fish in X and Y direction.
    * **speed_randomness** - A multiplier used when randomizing fish speed.
    Should be between 0.0 and 1.0.
    * **area** - The area this fish can move in. Max X is 100, max Y is 62.5.

### Example

```json
{
    "input_data_path": "assets/inputdata.json",
    "display_time": 10,
    "backgrounds": [
        "assets/background.png",
    ],
    "water_sprite": "water.png",
    "scenes": [
        {
            "input_data_path": "inputdata.json",
            "display_time": 5,
            "background": 3
        },
        {
            "input_data_path": "inputdata2.json",
            "display_time": 5,
            "background": 2,
            "legend": {
                "description": "These are the different fishes in this aquarium.",
                "fish_legends": [
                    {
                        "fish": "crab",
                        "description": "A happy rustacean!"
                    }
                ]
            }
        }
    ],
    "fishes": {
        "crab": {
            "texture": "assets/ferris.png",
            "size": 7.0,
            "size_randomness": 1.0,
            "movement": "Crab",
            "bubbles": 0,
            "collision_aversion": 0.3,
            "speed": {
                "x": 12.0,
                "y": 4.0
            },
            "speed_randomness": {
                "x": 1.0,
                "y": 1.0
            },
            "area": {
                "x": 5.0,
                "y": 56.0,
                "w": 90.0,
                "h": 6.0
            }
        },
    }
}
```

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

Related links
=============

* <a rel="me" href="https://hachyderm.io/@ollej">ollej @ mastodon</a>
* [Rusty Slider](https://ollej.github.io/rusty-slider/)
* [Olle's portfolio](https://olle.wreede.se/)
