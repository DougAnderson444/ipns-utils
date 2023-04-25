use anyhow::Result;
use ipns_server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let _handle = tokio::spawn(async {
        // let _result = ipns_server::start().await.unwrap();
        let _ = Server::new()
            .enable_kademlia()
            .enable_autonat()
            .start_with_tokio_executor()
            .await;
    });

    println!("\n*** To Shutdown, use Ctrl + C ***\n");

    match tokio::signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {err}");
            // we also shut down in case of error
        }
    };

    Ok(())
}
