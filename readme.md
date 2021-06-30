# SAMP CEF
This project embeds CEF into SA:MP expanding abilities to express yourself with beauty in-game interfaces using HTML / CSS / JavaScript.

**THIS MAYBE NOT PRODUCTION READY (THERE IS ONLY ONE SERVER USING IT RIGHT NOW)**

**It is a FRAMEWORK (or SDK), not something that you download and use**

## What you can do
- Create browser views from a gamemode or from client-side plugins (C ABI).
- Place browsers on objects (with kind-of spatial sound)
- Send and receive custom defined events from / to clients.

## Building
### Dependencies
- [Rust compiler (nightly) with `i686-windows-pc-msvc` toolchain](https://rust-lang.org)
- Prebuilt CEF with proprietary codes (if you wanna use streams). I had one for you in releases.
- Microsoft DirectX SDK (June 2010)

I am a bad man. So ... You should change hard-coded links in the source code ...

- `client/build.rs` - path to DirectX SDK (default one)
- `cef-sys/build.rs` - path to a static CEF library (libcef.lib).

and now
> `cargo +nightly build --target i686-windows-pc-msvc --release`

Current versions of CEF and Chromium:
`89.0.5+gc1f90d8+chromium-89.0.4389.40` `release branch 4389`

```
Date:             February 26, 2021

CEF Version:      89.0.5+gc1f90d8+chromium-89.0.4389.40
CEF URL:          https://bitbucket.org/chromiumembedded/cef.git
                  @c1f90d8c933dce163b74971707dbd79f00f18219

Chromium Version: 89.0.4389.40
Chromium URL:     https://chromium.googlesource.com/chromium/src.git
                  @2c3400a2b467aa3cf67b4942740db29e60feecb8
```

docs (ru only sorry ......) [docs/main_ru.md](/docs/main_ru.md)

видеоприколы на основе разработки:
- https://www.youtube.com/watch?v=Jh9IBlOKoVM (гоблин на весь дом)
- https://www.youtube.com/watch?v=jU-O8_t1AfI (простые интерфейсы)
- https://www.youtube.com/watch?v=qs7n8LoVYs4 (кастомный интерфейс гта)
- https://www.youtube.com/watch?v=vcyTjn3RJhs (голосовой чят)
- https://www.youtube.com/watch?v=6OnCSHKcOGU (кухня по телеку)
