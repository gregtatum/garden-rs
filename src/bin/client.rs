use futures::executor::block_on;
use futures::prelude::*;
use libp2p::multiaddr::Protocol;
use libp2p::ping::{Ping, PingConfig};
use libp2p::swarm::SwarmEvent;
use libp2p::{identity, Multiaddr, PeerId, Swarm};
use std::task::Poll;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "client", about = "A client for the blockchain")]
struct CliOptions {
    #[structopt(long)]
    ip: Option<String>,
    #[structopt(long)]
    port: Option<String>,
}

fn main() {
    dotenv::dotenv().ok();
    let cli_options = CliOptions::from_args();

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport =
        block_on(libp2p::development_transport(local_key)).expect("Failed to create a transport");

    let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));

    let mut swarm = Swarm::new(transport, behaviour, local_peer_id);

    dotenv::dotenv().expect("Failed to load the .env file.");

    // Get the self address from .env,
    let self_addr = Multiaddr::empty()
        .with(Protocol::Ip4(
            std::env::var("IP_ADDR")
                .unwrap_or_else(|_| String::from("0.0.0.0"))
                .parse()
                .expect("Failed to parse an IPv4 address."),
        ))
        .with(Protocol::Tcp(
            std::env::var("TCP_PORT")
                .unwrap_or_else(|_| String::from("0"))
                .parse()
                .expect("Failed to parse a TCP port."),
        ));

    // Attempt to dial a client.
    if let Some(ip) = cli_options.ip {
        let port = cli_options
            .port
            .expect("A --port is required when an --ip is specified.");

        let remote_addr: Multiaddr = Multiaddr::empty()
            .with(Protocol::Ip4(
                ip.parse()
                    .expect("Failed to parse the provided remote IP address"),
            ))
            .with(Protocol::Tcp(
                port.parse()
                    .expect("Failed to parse the provided remote port."),
            ));

        swarm
            .dial(remote_addr)
            .expect("Failed to dial remote address");

        println!("Dialed {}:{}", ip, port)
    }

    // Start listening on the swarm.
    swarm
        .listen_on(self_addr)
        .expect("Failed to listen on the swarm.");

    // Perform an event loop.
    block_on(future::poll_fn(move |cx| loop {
        match swarm.poll_next_unpin(cx) {
            Poll::Ready(Some(event)) => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(event) => {
                    println!("{:?}", event);
                }
                _ => {}
            },
            Poll::Ready(None) => return Poll::Ready(()),
            Poll::Pending => return Poll::Pending,
        }
    }));
}
