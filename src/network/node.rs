use libp2p:: {
    gossipsub, mdns, noise, tcp, yamux,
    swarm::NetworkBehaviour,
    identity::Keypair,
    PeerId,
    SwarmBuilder,
};
use std::hash::{Hash, Hasher, DefaultHasher};
use libp2p::Swarm;
use libp2p::swarm::SwarmEvent;
use libp2p::futures::StreamExt;
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};


pub enum NodeCommand {
    AddBlock(String),
    SubmitTx(crate::types::transaction::Transaction),
}

#[derive(Serialize, Deserialize)]
pub enum NetworkMessage {
    SyncRequest { height: u64, nonce: u64 },
    SyncResponse { blocks: Vec<crate::chain::block::Block>, nonce: u64 },
}


#[derive(NetworkBehaviour)]
pub struct NodeBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub struct Node {
    pub peer_id: PeerId,
    pub blockchain: crate::chain::blockchain::Blockchain,
    pub swarm: Swarm<NodeBehaviour>,
    pub topic: gossipsub::IdentTopic,
    pub sync_topic: gossipsub::IdentTopic,
    pub tx_topic: gossipsub::IdentTopic,
    pub cmd_rx: mpsc::Receiver<NodeCommand>,
    pub peers_connected: usize,
}

impl Node {

    // create a new Node instance
    pub async fn new(blockchain: crate::chain::blockchain::Blockchain) -> Result<(Self, mpsc::Sender<NodeCommand>) , String> {
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
            ).map_err(|e| e.to_string())?
            .with_behaviour(|key| {
                // gossipsub
                let message_id_fn = |message: &gossipsub::Message| { // dedup messages -> doesnt broadcast same 1 twice
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .message_id_fn(message_id_fn)
                    .mesh_outbound_min(0)// @todo REMOVE
                    .mesh_n_low(1) // @todo REMOVE: threshold for activley seeking more peers
                    .mesh_n(2) // @todo REMOVE
                    .mesh_n_high(4) // @todo REMOVE
                    .heartbeat_interval(std::time::Duration::from_millis(100))
                    .build()
                    .map_err(|e| e.to_string())?;
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config
                ).map_err(|e| e.to_string())?;

                //mdns
                let mdns = mdns::tokio::Behaviour::new( // broadcast and lsiten
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                ).map_err(|e| e.to_string())?;
                Ok(NodeBehaviour { gossipsub, mdns })
            }).map_err(|e| e.to_string())?
            .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
            .build();

        // subscribes to listening to blocks
        let topic = gossipsub::IdentTopic::new("blocks");
        swarm.behaviour_mut().gossipsub.subscribe(&topic).map_err(|e| e.to_string())?;

        // creates tokio channel with buffer of 32 msgs
        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        
        // creates topic for syncing chain
        let sync_topic = gossipsub::IdentTopic::new("sync");
        swarm.behaviour_mut().gossipsub.subscribe(&sync_topic).map_err(|e| e.to_string())?;

        // creates topic for syncing mempool
        let tx_topic = gossipsub::IdentTopic::new("transactions");
        swarm.behaviour_mut().gossipsub.subscribe(&tx_topic).map_err(|e| e.to_string())?;

        Ok( (Node { peer_id, blockchain, swarm, topic, sync_topic, tx_topic, cmd_rx, peers_connected: 0}, cmd_tx) )
    }

    // run the node
    pub async fn run(&mut self) -> Result<(), String> {
        // listen on a random port
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().map_err(|e: libp2p::multiaddr::Error| e.to_string())?).map_err(|e| e.to_string())?;

        loop {
            //  waits on 2 things at once
            tokio::select! {
                // waiting to hear new blocks added
                cmd = self.cmd_rx.recv() => {
                    if let Some(cmd) = cmd {
                        match cmd {
                            NodeCommand::AddBlock(miner) => {
                                match self.blockchain.add_block(&miner) {
                                    Ok(_) => {
                                        println!("Block Added");
                                        match self.blockchain.blocks.last() {
                                            Some(block) => {
                                                if self.peers_connected > 0 {
                                                    let block = block.clone();
                                                    if let Err(e) = self.broadcast_block(&block) {
                                                        println!("Broadcast failed {}", e);
                                                    }
                                                } else {
                                                    println!("No peers connected. Skipping boradcast");
                                                }
                                            }
                                            None => println!("No blocks to boradcast"),
                                        }
                                    }
                                    Err(e) => println!("Failed to add block: {}", e),
                                }
                            }
                            NodeCommand::SubmitTx(tx) => {
                                if let Err(e) = self.blockchain.submit_tx(tx.clone()) {
                                    println!("Failed to submit tx to mempool: {}", e);
                                } else {
                                    if let Err(e) = self.broadcast_tx(&tx) {
                                        println!("Failed to broadcast tx to mempool: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }

                event = self.swarm.select_next_some() => {
                    match event {
                        // successfully listneing on a port
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("Listening on {address}");
                        }
        
                        // found a peer on a local network
                        SwarmEvent::Behaviour(NodeBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                            for (peer_id, addr) in peers {
                                println!("Discovered peer: {peer_id}");
                                self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id); // add peer
                                
                                // creates subsrciption & avoids dialing collision w/ rand time
                                let peer_id_clone = peer_id;
                                let addr_clone = addr;
                                tokio::time::sleep(tokio::time::Duration::from_millis(
                                    rand::random::<u64>() % 1000
                                )).await;
                                self.swarm.dial(addr_clone).map_err(|e| e.to_string())?;
                            }
                        }
        
                        // peer disappeared -> remove them
                        SwarmEvent::Behaviour(NodeBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                            for (peer_id, _) in peers {
                                println!("Peer expired: {peer_id}");
                                self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                            }
                        }
        
                        // message recieved loud & clear
                        SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                            if message.topic == self.sync_topic.hash() {
                                //handle sync message
                                match bincode::deserialize::<NetworkMessage>(&message.data) {
                                    Ok(NetworkMessage::SyncRequest { height, .. }) => {
                                        println!("Peer request sync from height {height}");
                                        let our_height = self.blockchain.blocks.len() as u64;
                                        if our_height > height {
                                            // send missing blocks
                                            let missing: Vec<_> = self.blockchain.blocks[height as usize ..].to_vec();
                                            let msg = NetworkMessage::SyncResponse { 
                                                blocks: missing,
                                                nonce: rand::random::<u64>()
                                            };
                                            self.publish_sync(msg)?;
                                        }

                                    }
                                    Ok(NetworkMessage::SyncResponse { blocks, .. }) => {
                                        println!("Received {} blocks from sync", blocks.len());
                                        for block in blocks {
                                            if let Err(e) = self.blockchain.validate_and_add(&block) {
                                                println!("Block rejected: {} - requesting sync", e);
                                                break;
                                            }
                                        }
                                    }
                                    Err(e) => println!("Failed to deserialize sync message: {}", e),
                                }

                            } else if message.topic == self.topic.hash() {
                                // handle block broadcast
                                match bincode::deserialize::<crate::chain::block::Block>(&message.data) {
                                    Ok(block) => {
                                        println!("Received block {} from {:?}", block.index, message.source);
                                        if let Err(e) = self.blockchain.validate_and_add(&block) {
                                            println!("Block rejected: {}", e);
                                            // requests full sync
                                            let height = self.blockchain.blocks.len() as u64;
                                            let msg = NetworkMessage::SyncRequest { 
                                                height,
                                                nonce: rand::random::<u64>()
                                            };
                                            self.publish_sync(msg)?;
                                        }
                                    }
                                    Err(e) => println!("Failed to deserialize block: {}", e),
                                }
                            } else if message.topic == self.tx_topic.hash() {
                                // handle mempool broadcast
                                match bincode::deserialize::<crate::types::transaction::Transaction>(&message.data) {
                                    Ok(tx) => {
                                        println!("Received tx from {:?}", message.source);
                                        if let Err(e) = self.blockchain.submit_tx(tx) {
                                            println!("NODE: Failed to add tx to mempool: {}", e);
                                        }

                                    }
                                    Err(e) => println!("Failed to deserialize transaction: {}", e),
                                }
                            }
                        }

                        // peers are connected
                        SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed { peer_id, topic })) => {
                            println!("Peer {peer_id} subscribed to {topic}");
                            self.peers_connected += 1;

                            // tell chain height to new peer
                            let height = self.blockchain.blocks.len() as u64;
                            let msg = NetworkMessage::SyncRequest { 
                                height,
                                nonce: rand::random::<u64>()
                            };
                            self.publish_sync(msg)?;
                        }

                        // peer is disconnected
                        SwarmEvent::Behaviour(NodeBehaviourEvent::Gossipsub(gossipsub::Event::Unsubscribed { peer_id, .. })) => {
                            println!("Peer {peer_id} unsubscribed");
                            if self.peers_connected > 0 { self.peers_connected -= 1; }
                        }

                        _ => {}
                    }
    
                }
            }
        }
    }

    // searlizes the block and broadcasts it to the network
    pub fn broadcast_block(&mut self, block: &crate::chain::block::Block) -> Result<(), String> {
        let encoded = bincode::serialize(block).map_err(|e| e.to_string())?;
        match self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), encoded) { // @todo : implement retry queue
                Ok(_) => Ok(()),
                Err(gossipsub::PublishError::InsufficientPeers) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
    }

    // adds block then broadcasts it
    pub fn add_and_broadcast(&mut self, miner: &str) -> Result<(), String> {
        self.blockchain.add_block(miner)?;
        let block = self.blockchain.blocks.last().ok_or("no blocks")?.clone();
        self.broadcast_block(&block)?;
        Ok(())
    }

    // request sync of chain from peer
    fn publish_sync(&mut self, msg:NetworkMessage) -> Result<(), String> {
        let encoded = bincode::serialize(&msg).map_err(|e| e.to_string())?;
        match self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.sync_topic.clone(), encoded) { // @todo : implement retry queue
                Ok(_) => Ok(()),
                Err(gossipsub::PublishError::InsufficientPeers) => Ok(()),
                Err(gossipsub::PublishError::Duplicate) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
    }

    // broadcasts tx to mempool
    pub fn broadcast_tx(&mut self, tx: &crate::types::transaction::Transaction) -> Result<(), String> {
        let encoded = bincode::serialize(tx).map_err(|e| e.to_string())?;
        match self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.tx_topic.clone(), encoded) {
                Ok(_) => Ok(()),
                Err(gossipsub::PublishError::InsufficientPeers) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
    }
}