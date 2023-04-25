use libp2p::autonat;
use libp2p::gossipsub;
use libp2p::identify;
use libp2p::identity::Keypair;
use libp2p::kad::{record::store::MemoryStore, Kademlia, KademliaConfig, KademliaEvent};
use libp2p::ping;
use libp2p::relay;
use libp2p::swarm::{behaviour::toggle::Toggle, keep_alive, NetworkBehaviour};
use libp2p::{identity, Multiaddr, PeerId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::time::Duration;
use void::Void;

const BOOTNODES: [&str; 4] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
];

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event", event_process = false)]
pub struct Behaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: Toggle<Kademlia<MemoryStore>>,
    relay: relay::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    autonat: Toggle<autonat::Behaviour>,
    keep_alive: keep_alive::Behaviour,
}

impl Behaviour {
    pub fn new(id_keys: Keypair, enable_kademlia: bool, enable_autonat: bool) -> Self {
        let pub_key: identity::PublicKey = id_keys.public();

        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };

        // Set a custom gossipsub configuration
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            // .mesh_n_low(2) // experiment to see if this matters for WebRTC
            // .support_floodsub()
            .check_explicit_peers_ticks(1)
            .heartbeat_initial_delay(Duration::from_secs(30))
            .heartbeat_interval(Duration::from_secs(60)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
            .build()
            .expect("Valid config");

        let kademlia = if enable_kademlia {
            let mut kademlia_config = KademliaConfig::default();
            // Instantly remove records and provider records.
            //
            // TODO: Replace hack with option to disable both.
            kademlia_config.set_record_ttl(Some(Duration::from_secs(0)));
            kademlia_config.set_provider_record_ttl(Some(Duration::from_secs(0)));
            kademlia_config.set_query_timeout(Duration::from_secs(5 * 60));
            let mut kademlia = Kademlia::with_config(
                pub_key.to_peer_id(),
                MemoryStore::new(pub_key.to_peer_id()),
                kademlia_config,
            );
            let bootaddr = Multiaddr::from_str("/dnsaddr/bootstrap.libp2p.io").unwrap();
            for peer in &BOOTNODES {
                kademlia.add_address(&PeerId::from_str(peer).unwrap(), bootaddr.clone());
            }
            kademlia.bootstrap().unwrap();
            kademlia.get_closest_peers(
                PeerId::from_str("QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN").unwrap(),
            );

            Some(kademlia)
        } else {
            None
        }
        .into();

        let autonat = if enable_autonat {
            Some(autonat::Behaviour::new(
                PeerId::from(pub_key.clone()),
                Default::default(),
            ))
        } else {
            None
        }
        .into();

        Self {
            autonat,
            kademlia,
            keep_alive: keep_alive::Behaviour,
            relay: relay::Behaviour::new(PeerId::from(pub_key.clone()), Default::default()),
            ping: ping::Behaviour::new(ping::Config::new()),
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(id_keys),
                gossipsub_config,
            )
            .expect("Valid configuration"),
            identify: identify::Behaviour::new(
                identify::Config::new("ipfs/0.1.0".to_string(), pub_key).with_agent_version(
                    format!("rust-libp2p-server/{}", env!("CARGO_PKG_VERSION")),
                ),
            ),
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Gossipsub(gossipsub::Event),
    Ping(ping::Event),
    Identify(Box<identify::Event>),
    Relay(relay::Event),
    Kademlia(KademliaEvent),
    Autonat(autonat::Event),
}

impl From<gossipsub::Event> for Event {
    fn from(event: gossipsub::Event) -> Self {
        Event::Gossipsub(event)
    }
}

impl From<ping::Event> for Event {
    fn from(event: ping::Event) -> Self {
        Event::Ping(event)
    }
}

impl From<identify::Event> for Event {
    fn from(event: identify::Event) -> Self {
        Event::Identify(Box::new(event))
    }
}

impl From<relay::Event> for Event {
    fn from(event: relay::Event) -> Self {
        Event::Relay(event)
    }
}

impl From<KademliaEvent> for Event {
    fn from(event: KademliaEvent) -> Self {
        Event::Kademlia(event)
    }
}

impl From<autonat::Event> for Event {
    fn from(event: autonat::Event) -> Self {
        Event::Autonat(event)
    }
}

impl From<Void> for Event {
    fn from(event: Void) -> Self {
        void::unreachable(event)
    }
}
