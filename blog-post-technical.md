Rusty Aquarium technical description
====================================

![Rusty Aquarium](https://ollej.github.io/rusty-aquarium/assets/open-graph-preview.png)

Rusty Aquarium is a data visualization tool that shows analytics data as fish
swimming in an aquarium.

## How Rusty Aquarium works

A json file with input data is read periodically, which defines which fish
are to be shown. This file only includes the type of fish, and multipliers for
speed, size, and movement bubbles. It’s also possible to define which of a
predefined list of background images to show.

The type of fish is matched to the configured fish, to find out which image to
show, and what the default speed and size are.

Different “scenes” can be simulated by returning a different set of input data
together with a different background each time the input data file is loaded.

## Configuration

The file `assets/config.json` can be used to configure the fish tank. It has
some general configuration values, paths to PNG files to load for background
images and fish sprites. It also defines which fish types are available,
with default values for them.

### File format

```json
{
    "data_reload_time": 10,
    "background_switch_time": 0,
    "backgrounds": [
        "assets/background.png",
    ],
    "fishes": {
        "crab": {
            "texture": "assets/ferris.png",
            "size": 7.0,
            "size_randomness": 1.0,
            "movement": { "Crab": [] },
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

 * **data_reload_time** - Reload `inputdata.json` after this number of
 seconds. Set to `0` to never reload data.
 * **background_switch_time** - Number of seconds each background image is
 shown. Set to `0` to never change the background automatically. It can still
 be updated by `inputdata.json`.
 * **backgrounds** - An array of strings with paths to PNG images to use as
 background images.
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

## Generate input data

The file `assets/inputdata.json` is read to define what fish to display.

The field `background` is optional, and can be used to select which
background to show. The number should match the index of the wanted
background from the `backgrounds` field in `config.json`.

The field `school` must be an array of objects, one for each fish to
display.

The `fish` field must match a name in the map of fishes in the
`config.json` file.

The fields `size`, `speed`, and `bubbles` are multipliers that will be applied
to the corresponding value from the fish configuration.

### File format

```json
{
    "background": 1,
    "school": [
        { "fish": "crab", "size": 1.0, "speed": 1.0, "bubbles": 1.0 },
    ]
}
```

## Tech stack

The code is written in the Rust programming language. For the web version, it
is compiled into web assembly that draws on a browser canvas using WebGL. It
can also be compiled natively for different computer platforms. 

The graphics are handled by the Macroquad, which is a simple and easy to use
game library for Rust.

## Contributing

If you would like to contribute to Rusty Aquarium, there are several ways to
go about it.

Adding additional movement patterns for fishes is a simple way to start if you
want to contribute code.

To implement something bigger, maybe try implementing a new type of data
point, like a fractal layer of growing algae, or water bubbles rising from a
pump at the bottom.

It would also be nice to have a better OpenGL shader that's more appropriate
for the fish tank.

It is also much appreciated to have new types of fish. Send in the
configuration and image to represent the fish.

Check out the "help wanted" tag on the GitHub issues page to see other things
to help out with.
