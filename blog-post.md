Rusty Aquarium
==============

![Rusty Aquarium](https://ollej.github.io/rusty-aquarium/assets/screenshot.png)

## Regular old data visualization

We’ve all had bar graphs and line graphs showing the status of our systems. Data points are mapped to business events or just something completely irrelevant. These visualizations tend to look static, technical and BOOORING.

## Use an aquarium instead

Rusty Aquarium is an application that visualizes data points as fish swimming
around in an aquarium. It can be used to display data on a screen hanging in
your office. The data points can be mapped to different species of fish, as
well as their speed, size and movement bubbles.

Due to the nature of the fish tank not showing exact data, even sensitive
information can be shown in a public area. Visitors who don’t know the
specifics of what the fish represent will not see anything other than a serene
aquarium.

When you show data in this way people tend to get invested. You could choose
fish species to represent data from their business area and whenever they walk
past the aquarium they look at it to see if “their” fishes show up.

## How the aquarium works

A config file with input data is read periodically, which defines which fish
are to be shown. This file only includes the type of fish, and multipliers for
speed, size, and movement bubbles. It’s also possible to define which of a
predefined list of background images to show.

The type of fish is matched to the configured fish, to find out which image to
show, and what the default speed and size are.

Different “scenes” can be simulated by returning a different set of input data
together with a different background each time the input data file is loaded.

## Prepare your aquarium

The easiest way to run Rusty Aquarium is to publish the wasm files on a web
server. Then it’s just a matter of setting up a program that updates the
inputdata.json file, or generates it on the fly. To display the aquarium, run
a web browser in kiosk mode.

To use the aquarium as a screensaver, there are programs that can display a
URL when it activates.

## Gather the input data

The file `assets/inputdata.json` is read to define what fish to display.

The field background is optional, and can be used to select which background
to show. The number should match the index of the wanted background from the
backgrounds field in config.json.

The field `school` must be an array of objects, one for each fish to display.

The `fish` field must match a name in the map of fishes in the `config.json`
file.

The fields `size`, `speed`, and `bubbles` are multipliers that will be applied
to the corresponding value from the fish configuration.

## What else can it be used for?

### System monitoring

![System monitoring](https://ollej.github.io/rusty-aquarium/assets/screenshot-systemdata.png)

An example included with the application will show system monitoring data.
Each CPU is represented by a fish, and the usage will determine its size.
Every process is represented by a small fish, as well as usage each disk.

### Web request tracking

One possible usage could be to represent requests to a web server. Maybe each
important page is represented by a species of fish, the response time
corresponds to the speed, and the body length represents the size of the fish.

Specific events may also show up as separate fishes, such as logins or a
unique visitor.

### E-commerce metrics

In an e-commerce business each sale could represent a new fish, with the total
amount corresponding to the size of the fish. The species might represent the
category of the sale. Maybe the distance of the delivery could correspond to
the speed of the fish.

## Learn more

![Rust](https://ollej.github.io/rusty-aquarium/assets/ferris.png)

Rusty Aquarium is developed with the Rust programming language using the
Macroquad game engine. It can be compiled to run natively on Windows, Mac and
Linux, as well as web assembly that can run in the browser.

* [Rusty Aquarium on GitHub](https://github.com/ollej/rusty-aquarium)
* [Macroquad](https://macroquad.rs)

