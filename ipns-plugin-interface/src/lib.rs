use anyhow::{Error, Result};
use extism::Context;
use extism::Function;
use extism::ValType;
use extism::{CurrentPlugin, UserData, Val};
use extism::{Plugin, PluginBuilder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash)]
pub struct Output {
    pub count: i32,
    pub config: String,
    pub a: String,
}

pub struct IPNSPlugin<'a> {
    plugin: Plugin<'a>,
    context: Option<&'a Context>,
}

impl IPNSPlugin<'_> {
    pub fn new(wasm: &[u8]) -> Result<IPNSPlugin> {
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

        let f = Function::new(
            "hello_world",
            [ValType::I64],
            [ValType::I64],
            None,
            hello_world,
        );

        let context = None; // Context::new();

        let plugin = PluginBuilder::new_with_module(wasm)
            .with_wasi(true)
            .with_function(f)
            .build(context)?;

        Ok(IPNSPlugin { plugin, context })
    }

    pub fn with_config(
        &mut self,
        config: &std::collections::BTreeMap<String, Option<String>>,
    ) -> Result<&mut Self> {
        self.plugin.set_config(config)?;
        Ok(self)
    }

    /// Count the number of vowels in the input string
    pub fn count_vowels(&mut self, input: impl AsRef<[u8]>) -> Result<Output> {
        let data = self.plugin.call("count_vowels", input)?;

        // convert bytes to string
        let data_str = std::str::from_utf8(data)?;

        match serde_json::from_str(data_str) {
            Ok(d) => Ok(d),
            Err(e) => Err(Error::msg(format!("Error: {:?}", e))),
        }
    }
}

#[cfg(test)]
mod plugin_tests {
    use super::{IPNSPlugin, Output};
    use anyhow::{Error, Result};
    use extism::Context;
    use extism::Function;
    use extism::Plugin;
    use extism::ValType;
    use extism::{CurrentPlugin, UserData, Val};

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

        let thing = "this".to_string();
        let mut config = std::collections::BTreeMap::new();
        config.insert("thing".to_string(), Some(thing.to_owned()));

        let mut plugin = IPNSPlugin::new(wasm)?;
        plugin.with_config(&config)?;
        let data = plugin.count_vowels("this is a test")?;

        eprintln!("{data:?}");

        assert_eq!(data.count, 4);
        assert_eq!(data.config, thing);
        assert_eq!(data.a, "this is var a!");

        Ok(())
    }
}
