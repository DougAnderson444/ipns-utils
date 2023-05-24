# Plugin

Builds IPNS as a Extism plugin. This needs to be in it's own crate as it's `crate-type` is `cdylib` whereas the interface needs to be a `lib`, which can't be in the same crate.

## Test

Run the `ipns-interop-test` to test the plugin.
