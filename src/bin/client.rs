use futures::prelude::*;
use garden::chain_store::ChainStore;
use libp2p::{
    core::upgrade,
    floodsub::{self, Floodsub, FloodsubEvent},
    identity,
    mdns::{Mdns, MdnsEvent},
    mplex, noise,
    swarm::{NetworkBehaviourEventProcess, SwarmBuilder, SwarmEvent},
    tcp::TokioTcpConfig,
    Multiaddr, NetworkBehaviour, PeerId, Transport,
};
use std::{error::Error, path::PathBuf};
use structopt::StructOpt;
use tokio::io::{self, AsyncBufReadExt};

#[derive(Debug, StructOpt)]
#[structopt(name = "client", about = "A client for the blockchain")]
struct CliOptions {
    /// Multi-address to listen on.
    #[structopt(long, default_value = "/ip4/0.0.0.0/tcp/0")]
    listen_on: String,

    /// Multi-address to connect to, e.g. "/ip4/1.2.3.4/tcp/5678"
    #[structopt(long)]
    connect_to: Option<String>,

    /// The directory the garden files are persisted to.
    #[structopt(parse(from_os_str), default_value = "./.garden")]
    save_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli_options = CliOptions::from_args();

    let object_store = ChainStore::try_new(cli_options.save_path);

    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Create a tokio-based TCP transport use noise for authenticated
    // encryption and Mplex for multiplexing of substreams on a TCP stream.
    let transport = TokioTcpConfig::new()
        .nodelay(true)
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseConfig::xx({
                noise::Keypair::<noise::X25519Spec>::new()
                    .into_authentic(&local_key)
                    .expect("Signing libp2p-noise static DH keypair failed.")
            })
            .into_authenticated(),
        )
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    // Create a Floodsub topic
    let floodsub_topic = floodsub::Topic::new("chat");

    // We create a custom network behaviour that combines floodsub and mDNS.
    // The derive generates a delegating `NetworkBehaviour` impl which in turn
    // requires the implementations of `NetworkBehaviourEventProcess` for
    // the events of each behaviour.
    #[derive(NetworkBehaviour)]
    #[behaviour(event_process = true)]
    struct MyBehaviour {
        floodsub: Floodsub,
        mdns: Mdns,
    }

    impl NetworkBehaviourEventProcess<FloodsubEvent> for MyBehaviour {
        // Called when `floodsub` produces an event.
        fn inject_event(&mut self, message: FloodsubEvent) {
            if let FloodsubEvent::Message(message) = message {
                println!(
                    "Received: '{:?}' from {:?}",
                    String::from_utf8_lossy(&message.data),
                    message.source
                );
            }
        }
    }

    impl NetworkBehaviourEventProcess<MdnsEvent> for MyBehaviour {
        // Called when `mdns` produces an event.
        fn inject_event(&mut self, event: MdnsEvent) {
            match event {
                MdnsEvent::Discovered(list) => {
                    for (peer, _) in list {
                        self.floodsub.add_node_to_partial_view(peer);
                    }
                }
                MdnsEvent::Expired(list) => {
                    for (peer, _) in list {
                        if !self.mdns.has_node(&peer) {
                            self.floodsub.remove_node_from_partial_view(&peer);
                        }
                    }
                }
            }
        }
    }

    // Create a Swarm to manage peers and events.
    let mut swarm = {
        let mut behaviour = MyBehaviour {
            floodsub: Floodsub::new(local_peer_id.clone()),
            mdns: Mdns::new(Default::default()).await?,
        };

        behaviour.floodsub.subscribe(floodsub_topic.clone());

        SwarmBuilder::new(transport, behaviour, local_peer_id)
            // We want the connection background tasks to be spawned
            // onto the tokio runtime.
            .executor(Box::new(|future| {
                tokio::spawn(future);
            }))
            .build()
    };

    // Attempt to dial a client.
    if let Some(connect_to) = cli_options.connect_to {
        let addr: Multiaddr = connect_to.parse()?;
        println!("Dialing {}", addr);
        swarm.dial(addr).expect("Failed to dial remote address");
    }

    // Start listening on the swarm.
    swarm
        .listen_on(cli_options.listen_on.parse()?)
        .expect("Failed to listen on the swarm.");

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Event loop
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                swarm.behaviour_mut().floodsub.publish(floodsub_topic.clone(), line.as_bytes());
            }
            event = swarm.select_next_some() => {
                if let SwarmEvent::NewListenAddr { address, .. } = event {
                    println!("Listening on {:?}", address);
                }
            }
        }
    }
}
