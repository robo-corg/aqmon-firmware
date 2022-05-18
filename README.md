# Air quality monitor

This is mostly just me tinkering with the [PMS5003](https://www.aqmd.gov/docs/default-source/aq-spec/resources-page/plantower-pms5003-manual_v2-3.pdf) + ESP32C3. Right now all it does is dump the PMS5003 data out.

## Wiring the PMS5003

Hopefully your PMS5003 has a breakout for its odd sized serial cable. If it does wiring it up is fairly easy:

|PMS5003 Pin| ESP32-C3 Pin | Comment                                     |
|-----------|--------------|---------------------------------------------|
| VCC       | 5v           | PMS5003 runs on 5v but serial pins are 3.3v |
| GND       | GND          |                                             |
| TXD       | GPIO4        | GPIO4 is the pin 11 on the right            |
| RXD       | GPIO5        | Optional not sending anything to the PMS    |

## Building

### Prerequisites

You need nightly rust + some assort tooling for flashing the esp32c3:

```
# Install cli tools for dealing with esp32
cargo install -f ldproxy espflash espmonitor cargo-espflash

# Nightly toolchain is needed currently for the esp32c3 std support
rustup install nightly
rustup component add rust-src --toolchain nightly
```

### Build

Building should "just work":

```
cargo build
```

### Flashing

```
cargo espflash
```

Note: if you get permissions errors on linux make sure you are on the dialout

## Sourcing the parts

### ESP32-C3

I purchased my ESP32-C3-DevKitM-1's through sparkfun anywhere that sells these
should do. Not sure how compatible the unofficial non-espressif versions are
with rust.

### PMS5003

I originally purchased mine on https://www.adafruit.com/product/3686 but they
are often sold out. If you get one from amazon or some other sell make sure it
includes the cable + breakout board (or buy them separate).
