//! # ipns-interop-test
//!
//! Utilities that set up a test environment for IPNS interop tests.
use cmd_lib::*;

/// fn to check whether go-ipfs is installed or not
pub fn check_go_ipfs() -> FunResult {
    init_builtin_logger();
    cmd_lib::set_pipefail(false); // do not fail due to pipe errors

    let mut proc = spawn_with_output!(ipfs "--version")?;
    proc.wait_with_output()
}

#[cfg(test)]
mod go_interop_tests {
    use super::*;

    #[test]
    fn test_check_go_ipfs() {
        assert_eq!(check_go_ipfs().ok(), Some("ipfs version 0.18.1".to_owned()))
    }
}

#[cfg(test)]
mod plugin_tests {
    use anyhow::{Error, Result};
    use extism::Context;
    use extism::Function;
    use extism::Plugin;
    use extism::ValType;
    use extism::{CurrentPlugin, UserData, Val};
    use ipns_plugin_interface::Output;

    /// Host function that is called by the plugin
    // https://github.com/extism/rust-pdk/blob/main/examples/host_function.rs
    pub fn hello_world(
        _plugin: &mut CurrentPlugin,
        inputs: &[Val],
        outputs: &mut [Val],
        _user_data: UserData,
    ) -> Result<(), Error> {
        eprintln!("This is a host function!");
        outputs[0] = inputs[0].clone();
        Ok(())
    }

    #[test]
    fn it_works() -> Result<()> {
        let wasm = include_bytes!("../../target/wasm32-wasi/release/ipns_plugin_bindings.wasm");

        let context = Context::new();
        let f = Function::new(
            "hello_world",
            [ValType::I64],
            [ValType::I64],
            None,
            hello_world,
        );
        let thing = "this".to_string();
        let mut config = std::collections::BTreeMap::new();
        config.insert("thing".to_string(), Some(thing.to_owned()));

        let wasi = true;
        let mut plugin = Plugin::new(&context, wasm, [f], wasi)?.with_config(&config)?;

        let data = plugin.call("count_vowels", "this is a test").unwrap();

        // convert bytes to string
        let data_str = std::str::from_utf8(data).unwrap();
        eprintln!("{data_str:?}");

        let d: Output = serde_json::from_str(data_str).unwrap();

        assert_eq!(d.count, 4);
        assert_eq!(d.config, thing);
        assert_eq!(d.a, "this is var a!");

        Ok(())
    }
}
