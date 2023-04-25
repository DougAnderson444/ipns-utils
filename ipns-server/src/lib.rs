use anyhow::Result;
use futures::stream::StreamExt;
use futures_timer::Delay;
use libp2p::identify;
use libp2p::identity;
use libp2p::identity::PeerId;
use libp2p::kad;
use libp2p::metrics::{Metrics, Recorder};
use libp2p::multiaddr::{Multiaddr, Protocol};
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use log::{debug, info};
use prometheus_client::metrics::info::Info;
use prometheus_client::registry::Registry;
use std::error::Error;
use std::net::Ipv6Addr;
use std::path::PathBuf;
use std::str::FromStr;
use std::task::Poll;
use std::time::Duration;
use zeroize::Zeroizing;

mod behaviour;
mod config;
mod metric_server;
mod transport;

const BOOTSTRAP_INTERVAL: Duration = Duration::from_secs(5 * 60);
const PORT_WEBRTC: u16 = 9090;
const PORT_QUIC: u16 = 9091;
const PORT_TCP: u16 = 9092;

pub struct Server {
    /// Path to IPFS config file.
    config: Option<PathBuf>,

    /// Metric endpoint path.
    metrics_path: String,

    /// Whether to run the libp2p Kademlia protocol and join the IPFS DHT.
    enable_kademlia: bool,

    /// Whether to run the libp2p Autonat protocol.
    enable_autonat: bool,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Server {
            config: None,
            metrics_path: "/metrics".to_string(),
            enable_kademlia: false,
            enable_autonat: false,
        }
    }

    pub fn enable_kademlia(&mut self) -> &mut Server {
        self.enable_kademlia = true;
        self
    }

    pub fn enable_autonat(&mut self) -> &mut Server {
        self.enable_autonat = true;
        self
    }

    pub async fn start_with_tokio_executor(&self) -> Result<(), Box<dyn Error>> {
        env_logger::init();

        let (local_peer_id, local_keypair) = match &self.config {
            Some(path) => {
                let config = Zeroizing::new(config::Config::from_file(path.as_path())?);

                let keypair = identity::Keypair::from_protobuf_encoding(&Zeroizing::new(
                    base64::decode(config.identity.priv_key.as_bytes())?,
                ))?;

                let peer_id = keypair.public().into();
                assert_eq!(
                    PeerId::from_str(&config.identity.peer_id)?,
                    peer_id,
                    "Expect peer id derived from private key and peer id retrieved from config to match."
                );

                (peer_id, keypair)
            }
            None => {
                let keypair = identity::Keypair::generate_ed25519();
                (keypair.public().into(), keypair)
            }
        };
        println!("Local peer id: {local_peer_id:?}");

        let transport = transport::create(local_keypair.clone()).await?;

        let behaviour =
            behaviour::Behaviour::new(local_keypair, self.enable_kademlia, self.enable_autonat);

        let mut swarm =
            SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build();

        let address_webrtc = Multiaddr::from(Ipv6Addr::UNSPECIFIED)
            .with(Protocol::Udp(PORT_WEBRTC))
            .with(Protocol::WebRTCDirect);

        let address_quic = Multiaddr::from(Ipv6Addr::UNSPECIFIED)
            .with(Protocol::Udp(PORT_QUIC))
            .with(Protocol::QuicV1);

        let address_tcp = Multiaddr::from(Ipv6Addr::UNSPECIFIED).with(Protocol::Tcp(PORT_TCP));

        swarm.listen_on(address_webrtc.clone()).expect("listen rtc");
        swarm.listen_on(address_quic.clone()).expect("listen quic");
        swarm.listen_on(address_tcp.clone()).expect("listen on tcp");

        let mut metric_registry = Registry::default();
        let metrics = Metrics::new(&mut metric_registry);
        let build_info = Info::new(vec![("version".to_string(), env!("CARGO_PKG_VERSION"))]);
        metric_registry.register(
            "build",
            "A metric with a constant '1' value labeled by version",
            build_info,
        );

        let p = self.metrics_path.clone();
        tokio::spawn(async move { metric_server::run(metric_registry, p).await });

        let mut bootstrap_timer = Delay::new(BOOTSTRAP_INTERVAL);

        loop {
            if let Poll::Ready(()) = futures::poll!(&mut bootstrap_timer) {
                bootstrap_timer.reset(BOOTSTRAP_INTERVAL);
                let _ = swarm
                    .behaviour_mut()
                    .kademlia
                    .as_mut()
                    .map(|k| k.bootstrap());

                let _ = swarm.behaviour_mut().kademlia.as_mut().map(|k| {
                    k.get_closest_peers(
                        PeerId::from_str("QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN")
                            .expect("legit peerid"),
                    )
                });
            }

            match swarm.next().await.expect("Swarm not to terminate.") {
                SwarmEvent::Behaviour(behaviour::Event::Identify(e)) => {
                    info!("{:?}", e);
                    metrics.record(&*e);

                    if let identify::Event::Received {
                        peer_id,
                        info:
                            identify::Info {
                                listen_addrs,
                                protocols,
                                ..
                            },
                    } = *e
                    {
                        if protocols
                            .iter()
                            .any(|p| p.as_bytes() == kad::protocol::DEFAULT_PROTO_NAME)
                        {
                            for addr in listen_addrs {
                                swarm
                                    .behaviour_mut()
                                    .kademlia
                                    .as_mut()
                                    .map(|k| k.add_address(&peer_id, addr));
                            }
                        }
                    }
                }
                SwarmEvent::Behaviour(behaviour::Event::Ping(e)) => {
                    debug!("{:?}", e);
                    metrics.record(&e);
                }
                SwarmEvent::Behaviour(behaviour::Event::Kademlia(e)) => {
                    println!("** KAD Evt: {e:?}");
                    debug!("{:?}", e);
                    metrics.record(&e);
                }
                SwarmEvent::Behaviour(behaviour::Event::Relay(e)) => {
                    info!("{:?}", e);
                    metrics.record(&e)
                }
                SwarmEvent::Behaviour(behaviour::Event::Autonat(e)) => {
                    info!("{:?}", e);
                    // TODO: Add metric recording for `NatStatus`.
                    // metrics.record(&e)
                }
                SwarmEvent::Behaviour(behaviour::Event::Gossipsub(
                    libp2p::gossipsub::Event::Message { message, .. },
                )) => {
                    info!(
                        "📨 Received message from {:?}: {}",
                        message.source,
                        String::from_utf8(message.data).unwrap()
                    );
                }
                SwarmEvent::Behaviour(behaviour::Event::Gossipsub(
                    libp2p::gossipsub::Event::Subscribed { peer_id, topic },
                )) => {
                    info!("💨💨💨  {peer_id} subscribed to {topic}");
                }
                e => {
                    if let SwarmEvent::NewListenAddr { address, .. } = &e {
                        println!("Listening on {address:?}");
                    }

                    metrics.record(&e)
                }
            }
        }
    }
}
