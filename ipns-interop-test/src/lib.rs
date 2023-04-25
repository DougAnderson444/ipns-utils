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
mod tests {
    use super::*;

    #[test]
    fn test_check_go_ipfs() {
        assert_eq!(check_go_ipfs().ok(), Some("ipfs version 0.18.1".to_owned()))
    }
}
