use std::{error::Error, time::Duration};

use libp2p::{
    Multiaddr, 
    noise, ping, mdns, 
    swarm::{SwarmEvent, NetworkBehaviour}, 
    tcp, yamux
};

use futures::prelude::*;
use tracing_subscriber::EnvFilter;

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    ping: ping::Behaviour,
    mdns: libp2p::mdns::tokio::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            let my_mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;

            Ok(MyBehaviour { ping: ping::Behaviour::default(), mdns: my_mdns })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();
    
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    
    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {addr}")
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => println!("Listeng on {address:?}"),
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    swarm.dial(multiaddr.clone())?;
                    println!("Connection to: {peer_id}");
                }
            },
            _ => {}
        }
    }
}
