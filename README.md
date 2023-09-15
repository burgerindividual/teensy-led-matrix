## Teensy 4.0 LED Matrix

The code that drives a shift-register-based LED matrix entirely in software.

This is a dumb idea. However, if it works, it would be really silly.

#### Building and Uploading
`cargo objcopy --release -- -O ihex target/out.hex && teensy_loader_cli --mcu=TEENSY40 -w target/out.hex`