use libp2p:: {
    gossipsub, mdns, noise, tcp, yamux,
    swarm::NetworkBehaviour,
    identity::Keypair,
    PeerId,
    SwarmBuilder,
};
use std::hash::{Hash, Hasher, DefaultHasher};
use libp2p::Swarm;
use tokio::io::{self, AsyncBufReadExt};
use libp2p::swarm::SwarmEvent;
use libp2p::futures::StreamExt;


#[derive(NetworkBehaviour)]
pub struct NodeBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub struct Node {
    pub peer_id: PeerId,
    pub blockchain: crate::chain::blockchain::Blockchain,
    pub swarm: Swarm<NodeBehaviour>,
}

impl Node {

    // create a new Node instance
    pub async fn new(blockchain: crate::chain::blockchain::Blockchain) -> Self {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());

        println!("Local peer id : {peer_id}");

        // gets p2p identity
        let mut swarm = SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new, // encrypts traffic
                yamux::Config::default, // multiple streams over one tcp connection
            ).unwrap()
            .with_behaviour(|key| {
                // gossipsub
                let message_id_fn = |message: &gossipsub::Message| { // dedup messages -> doesnt broadcast same 1 twice
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .message_id_fn(message_id_fn)
                    .build()
                    .unwrap();
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config
                ).unwrap();

                //mdns
                let mdns = mdns::tokio::Behaviour::new( // broadcast and lsiten
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                ).unwrap();
                Ok(NodeBehaviour { gossipsub, mdns })
            }).unwrap()
            .build();

        Node { peer_id, blockchain, swarm}
    }

    // run the node
    pub async fn run(&mut self) {
        // listen on a random port
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        loop {
            match self.swarm.next().await {
                // successfully listneing on a port
                Some(SwarmEvent::NewListenAddr { address, .. }) => {
                    println!("Listening on {address}");
                }

                // found a peer on a local network
                Some(SwarmEvent::Behaviour(NodeBehaviourEvent::Mdns(mdns::Event::Discovered(peers)))) => {
                    for (peer_id, addr) in peers {
                        println!("Discovered peer: {peer_id}");
                        self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                }

                // peer disappeared -> remove them
                Some(SwarmEvent::Behaviour(NodeBehaviourEvent::Mdns(mdns::Event::Expired(peers)))) => {
                    for (peer_id, _) in peers {
                        println!("Peer expired: {peer_id}");
                        self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                }

                // message recieved loud & clear
                Some(SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. }))) => {
                    println!("Received message from {:?}", message.source);
                }
                _ => {}
            }
        }
    }
}