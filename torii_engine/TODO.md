# TODO

- Finish the windowing system

# IDEAS

- A primary concern is the plugin ecosystem I want with this game engine and shared library compilation
  - An obvious option is to compile and import using cdylib (forces C ABI) or rdylib (unstable, apparently)
  - Another option, I'm considering more is through WASM plugins
    - Compile functions into WASM code modules, then import and run
    - I'm concerned about the performance impact however, need to compare native vs dylib vs WASM modules