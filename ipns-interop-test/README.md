# IPNS Utils Interop Test

A crate for testing IPNS interop with Go and JS. This crate is used in the [interop test](todo!).

-   Check if go-ipfs is installed:

    ```bash
    go version
    ```

-   Install and start a go-ipfs node:

    ```bash
    go get -u github.com/ipfs/go-ipfs
    go-ipfs init
    go-ipfs daemon
    ```

-   get the go-ipfs PeerId:
    ```bash
    ipfs id -f "<id>"
    ```
-   Subscribe to PeerId topic in Rust using ipns-utils:
    ```bash
    cargo run --example ipns-subscribe -- --peer-id <peer-id>
    ```
-   Publish an IPNS entry to go-ipfs:

    ```bash
    ipfs name publish /ipfs/QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N
    ```

-   Check that Rust received the entry on the topic channel.
