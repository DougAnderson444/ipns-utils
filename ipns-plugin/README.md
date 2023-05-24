# Plugin

Wrapper around Extism plugin.

This crate use the interfaces (`lib`) and the bindings (`cdylib`) to build a wrapper for the Extism plugin. This needs to be in it's own crate as it depends on `extism` which cannot be compiled with the bindings into wasm\* targets, and thus also cannot be in interfaces as bindings depends on interfaces as well.

## Test

Run the `ipns-interop-test` to test the plugin.
