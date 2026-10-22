# Resotool

## Building

After installing [Rust](https://rustup.rs/), you can compile Resotool as follows:

```shell
cargo xtask bundle resotool --release
```

The VST3 bundle is also copied to your user VST3 plugin directory, replacing any previously installed copy:

- macOS: `~/Library/Audio/Plug-Ins/VST3`
- Windows: `%LOCALAPPDATA%\Programs\Common\VST3`
- Linux: `~/.vst3`
