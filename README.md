# rust-stm32f4-discovery

Just a practice of [STM32F4 Discovery](https://www.st.com/ja/evaluation-tools/stm32f4discovery.html) with Rust.

This is built with template from https://github.com/rust-embedded/cortex-m-quickstart, mainly using [cortex-m](https://crates.io/crates/cortex-m) and [stm32f4](https://crates.io/crates/stm32f4) crate.

## Setup

Follow https://rust-embedded.github.io/book/intro/install.html (or https://tomoyuki-nakabayashi.github.io/book/intro/install.html in Japanese).

## Flash

Connect board to PC with USB.

Run

```
openocd
```

And run in the other terminal

```
cargo run
# cargo run --release
```

And enter `continue` twice.
