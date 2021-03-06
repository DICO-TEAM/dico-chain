// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Discovery mechanisms of Substrate.
//!
//! The `DiscoveryBehaviour` struct implements the `NetworkBehaviour` trait of libp2p and is
//! responsible for discovering other nodes that are part of the network.
//!
//! Substrate uses the following mechanisms in order to discover nodes that are part of the network:
//!
//! - Bootstrap nodes. These are hard-coded node identities and addresses passed in the constructor
//! of the `DiscoveryBehaviour`. You can also call `add_known_address` later to add an entry.
//!
//! - mDNS. Discovers nodes on the local network by broadcasting UDP packets.
//!
//! - Kademlia random walk. Once connected, we perform random Kademlia `FIND_NODE` requests on the
//! configured Kademlia DHTs in order for nodes to propagate to us their view of the network. This
//! is performed automatically by the `DiscoveryBehaviour`.
//!
//! Additionally, the `DiscoveryBehaviour` is also capable of storing and loading value in the
//! configured DHTs.
//!
//! ## Usage
//!
//! The `DiscoveryBehaviour` generates events of type `DiscoveryOut`, most notably
//! `DiscoveryOut::Discovered` that is generated whenever we discover a node.
//! Only the identity of the node is returned. The node's addresses are stored within the
//! `DiscoveryBehaviour` and can be queried through the `NetworkBehaviour` trait.
//!
//! **Important**: In order for the discovery mechanism to work properly, there needs to be an
//! active mechanism that asks nodes for the addresses they are listening on. Whenever we learn
//! of a node's address, you must call `add_self_reported_address`.
//!

use crate::config::ProtocolId;
use futures::prelude::*;
use futures_timer::Delay;
use ip_network::IpNetwork;
use libp2p::core::{connection::{ConnectionId, ListenerId}, ConnectedPoint, Multiaddr, PeerId, PublicKey};
use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters, ProtocolsHandler};
use libp2p::swarm::protocols_handler::multi::MultiHandler;
use libp2p::kad::{Kademlia, KademliaConfig, KademliaEvent, QueryResult, Quorum, Record};
use libp2p::kad::GetClosestPeersError;
use libp2p::kad::handler::KademliaHandler;
use libp2p::kad::QueryId;
use libp2p::kad::record::{self, store::{MemoryStore, RecordStore}};
#[cfg(not(target_os = "unknown"))]
use libp2p::swarm::toggle::Toggle;
#[cfg(not(target_os = "unknown"))]
use libp2p::mdns::{Mdns, MdnsEvent};
use libp2p::multiaddr::Protocol;
use log::{debug, info, trace, warn};
use std::{cmp, collections::{HashMap, HashSet, VecDeque}, io, time::Duration};
use std::task::{Context, Poll};
use sp_core::hexdisplay::HexDisplay;

/// `DiscoveryBehaviour` configuration.
///
/// Note: In order to discover nodes or load and store values via Kademlia one has to add at least
///       one protocol via [`DiscoveryConfig::add_protocol`].
pub struct DiscoveryConfig {
	local_peer_id: PeerId,
	user_defined: Vec<(PeerId, Multiaddr)>,
	allow_private_ipv4: bool,
	allow_non_globals_in_dht: bool,
	discovery_only_if_under_num: u64,
	enable_mdns: bool,
	kademlias: HashMap<ProtocolId, Kademlia<MemoryStore>>
}

impl DiscoveryConfig {
	/// Create a default configuration with the given public key.
	pub fn new(local_public_key: PublicKey) -> Self {
		DiscoveryConfig {
			local_peer_id: local_public_key.into_peer_id(),
			user_defined: Vec::new(),
			allow_private_ipv4: true,
			allow_non_globals_in_dht: false,
			discovery_only_if_under_num: std::u64::MAX,
			enable_mdns: false,
			kademlias: HashMap::new()
		}
	}

	/// Set the number of active connections at which we pause discovery.
	pub fn discovery_limit(&mut self, limit: u64) -> &mut Self {
		self.discovery_only_if_under_num = limit;
		self
	}

	/// Set custom nodes which never expire, e.g. bootstrap or reserved nodes.
	pub fn with_user_defined<I>(&mut self, user_defined: I) -> &mut Self
	where
		I: IntoIterator<Item = (PeerId, Multiaddr)>
	{
		for (peer_id, addr) in user_defined {
			for kad in self.kademlias.values_mut() {
				kad.add_address(&peer_id, addr.clone());
			}
			self.user_defined.push((peer_id, addr))
		}
		self
	}

	/// Should private IPv4 addresses be reported?
	pub fn allow_private_ipv4(&mut self, value: bool) -> &mut Self {
		self.allow_private_ipv4 = value;
		self
	}

	/// Should non-global addresses be inserted to the DHT?
	pub fn allow_non_globals_in_dht(&mut self, value: bool) -> &mut Self {
		self.allow_non_globals_in_dht = value;
		self
	}

	/// Should MDNS discovery be supported?
	pub fn with_mdns(&mut self, value: bool) -> &mut Self {
		if value && cfg!(target_os = "unknown") {
			log::warn!(target: "sub-libp2p", "mDNS is not available on this platform")
		}
		self.enable_mdns = value;
		self
	}

	/// Add discovery via Kademlia for the given protocol.
	pub fn add_protocol(&mut self, p: ProtocolId) -> &mut Self {
		// NB: If this protocol name derivation is changed, check if
		// `DiscoveryBehaviour::new_handler` is still correct.
		let proto_name = {
			let mut v = vec![b'/'];
			v.extend_from_slice(p.as_bytes());
			v.extend_from_slice(b"/kad");
			v
		};

		self.add_kademlia(p, proto_name);
		self
	}

	fn add_kademlia(&mut self, id: ProtocolId, proto_name: Vec<u8>) {
		if self.kademlias.contains_key(&id) {
			warn!(target: "sub-libp2p", "Discovery already registered for protocol {:?}", id);
			return
		}

		let mut config = KademliaConfig::default();
		config.set_protocol_name(proto_name);

		let store = MemoryStore::new(self.local_peer_id.clone());
		let mut kad = Kademlia::with_config(self.local_peer_id.clone(), store, config);

		for (peer_id, addr) in &self.user_defined {
			kad.add_address(peer_id, addr.clone());
		}

		self.kademlias.insert(id, kad);
	}

	/// Create a `DiscoveryBehaviour` from this config.
	pub fn finish(self) -> DiscoveryBehaviour {
		DiscoveryBehaviour {
			user_defined: self.user_defined,
			kademlias: self.kademlias,
			next_kad_random_query: Delay::new(Duration::new(0, 0)),
			duration_to_next_kad: Duration::from_secs(1),
			pending_events: VecDeque::new(),
			local_peer_id: self.local_peer_id,
			num_connections: 0,
			allow_private_ipv4: self.allow_private_ipv4,
			discovery_only_if_under_num: self.discovery_only_if_under_num,
			#[cfg(not(target_os = "unknown"))]
			mdns: if self.enable_mdns {
				match Mdns::new() {
					Ok(mdns) => Some(mdns).into(),
					Err(err) => {
						warn!(target: "sub-libp2p", "Failed to initialize mDNS: {:?}", err);
						None.into()
					}
				}
			} else {
				None.into()
			},
			allow_non_globals_in_dht: self.allow_non_globals_in_dht
		}
	}
}

/// Implementation of `NetworkBehaviour` that discovers the nodes on the network.
pub struct DiscoveryBehaviour {
	/// User-defined list of nodes and their addresses. Typically includes bootstrap nodes and
	/// reserved nodes.
	user_defined: Vec<(PeerId, Multiaddr)>,
	/// Kademlia requests and answers.
	kademlias: HashMap<ProtocolId, Kademlia<MemoryStore>>,
	/// Discovers nodes on the local network.
	#[cfg(not(target_os = "unknown"))]
	mdns: Toggle<Mdns>,
	/// Stream that fires when we need to perform the next random Kademlia query.
	next_kad_random_query: Delay,
	/// After `next_kad_random_query` triggers, the next one triggers after this duration.
	duration_to_next_kad: Duration,
	/// Events to return in priority when polled.
	pending_events: VecDeque<DiscoveryOut>,
	/// Identity of our local node.
	local_peer_id: PeerId,
	/// Number of nodes we're currently connected to.
	num_connections: u64,
	/// If false, `addresses_of_peer` won't return any private IPv4 address, except for the ones
	/// stored in `user_defined`.
	allow_private_ipv4: bool,
	/// Number of active connections over which we interrupt the discovery process.
	discovery_only_if_under_num: u64,
	/// Should non-global addresses be added to the DHT?
	allow_non_globals_in_dht: bool
}

impl DiscoveryBehaviour {
	/// Returns the list of nodes that we know exist in the network.
	pub fn known_peers(&mut self) -> HashSet<PeerId> {
		let mut peers = HashSet::new();
		for k in self.kademlias.values_mut() {
			for b in k.kbuckets() {
				for e in b.iter() {
					if !peers.contains(e.node.key.preimage()) {
						peers.insert(e.node.key.preimage().clone());
					}
				}
			}
		}
		peers
	}

	/// Adds a hard-coded address for the given peer, that never expires.
	///
	/// This adds an entry to the parameter that was passed to `new`.
	///
	/// If we didn't know this address before, also generates a `Discovered` event.
	pub fn add_known_address(&mut self, peer_id: PeerId, addr: Multiaddr) {
		if self.user_defined.iter().all(|(p, a)| *p != peer_id && *a != addr) {
			for k in self.kademlias.values_mut() {
				k.add_address(&peer_id, addr.clone());
			}
			self.pending_events.push_back(DiscoveryOut::Discovered(peer_id.clone()));
			self.user_defined.push((peer_id, addr));
		}
	}

	/// Call this method when a node reports an address for itself.
	///
	/// **Note**: It is important that you call this method, otherwise the discovery mechanism will
	/// not properly work.
	pub fn add_self_reported_address(&mut self, peer_id: &PeerId, addr: Multiaddr) {
		if self.allow_non_globals_in_dht || self.can_add_to_dht(&addr) {
			for k in self.kademlias.values_mut() {
				k.add_address(peer_id, addr.clone());
			}
		} else {
			log::trace!(target: "sub-libp2p", "Ignoring self-reported address {} from {}", addr, peer_id);
		}
	}

	/// Start fetching a record from the DHT.
	///
	/// A corresponding `ValueFound` or `ValueNotFound` event will later be generated.
	pub fn get_value(&mut self, key: &record::Key) {
		for k in self.kademlias.values_mut() {
			k.get_record(key, Quorum::One);
		}
	}

	/// Start putting a record into the DHT. Other nodes can later fetch that value with
	/// `get_value`.
	///
	/// A corresponding `ValuePut` or `ValuePutFailed` event will later be generated.
	pub fn put_value(&mut self, key: record::Key, value: Vec<u8>) {
		for k in self.kademlias.values_mut() {
			if let Err(e) = k.put_record(Record::new(key.clone(), value.clone()), Quorum::All) {
				warn!(target: "sub-libp2p", "Libp2p => Failed to put record: {:?}", e);
				self.pending_events.push_back(DiscoveryOut::ValuePutFailed(key.clone()));
			}
		}
	}

	/// Returns the number of nodes that are in the Kademlia k-buckets.
	pub fn num_kbuckets_entries(&mut self) -> impl ExactSizeIterator<Item = (&ProtocolId, usize)> {
		self.kademlias.iter_mut()
			.map(|(id, kad)| (id, kad.kbuckets().map(|bucket| bucket.iter().count()).sum()))
	}

	/// Returns the number of records in the Kademlia record stores.
	pub fn num_kademlia_records(&mut self) -> impl ExactSizeIterator<Item = (&ProtocolId, usize)> {
		// Note that this code is ok only because we use a `MemoryStore`.
		self.kademlias.iter_mut().map(|(id, kad)| {
			let num = kad.store_mut().records().count();
			(id, num)
		})
	}

	/// Returns the total size in bytes of all the records in the Kademlia record stores.
	pub fn kademlia_records_total_size(&mut self) -> impl ExactSizeIterator<Item = (&ProtocolId, usize)> {
		// Note that this code is ok only because we use a `MemoryStore`. If the records were
		// for example stored on disk, this would load every single one of them every single time.
		self.kademlias.iter_mut().map(|(id, kad)| {
			let size = kad.store_mut().records().fold(0, |tot, rec| tot + rec.value.len());
			(id, size)
		})
	}

	/// Can the given `Multiaddr` be put into the DHT?
	///
	/// This test is successful only for global IP addresses and DNS names.
	//
	// NB: Currently all DNS names are allowed and no check for TLD suffixes is done
	// because the set of valid domains is highly dynamic and would require frequent
	// updates, for example by utilising publicsuffix.org or IANA.
	pub fn can_add_to_dht(&self, addr: &Multiaddr) -> bool {
		let ip = match addr.iter().next() {
			Some(Protocol::Ip4(ip)) => IpNetwork::from(ip),
			Some(Protocol::Ip6(ip)) => IpNetwork::from(ip),
			Some(Protocol::Dns(_)) | Some(Protocol::Dns4(_)) | Some(Protocol::Dns6(_))
				=> return true,
			_ => return false
		};
		ip.is_global()
	}
}

/// Event generated by the `DiscoveryBehaviour`.
pub enum DiscoveryOut {
	/// The address of a peer has been added to the Kademlia routing table.
	///
	/// Can be called multiple times with the same identity.
	Discovered(PeerId),

	/// A peer connected to this node for whom no listen address is known.
	///
	/// In order for the peer to be added to the Kademlia routing table, a known
	/// listen address must be added via [`DiscoveryBehaviour::add_self_reported_address`],
	/// e.g. obtained through the `identify` protocol.
	UnroutablePeer(PeerId),

	/// The DHT yielded results for the record request, grouped in (key, value) pairs.
	ValueFound(Vec<(record::Key, Vec<u8>)>),

	/// The record requested was not found in the DHT.
	ValueNotFound(record::Key),

	/// The record with a given key was successfully inserted into the DHT.
	ValuePut(record::Key),

	/// Inserting a value into the DHT failed.
	ValuePutFailed(record::Key),

	/// Started a random Kademlia query for each DHT identified by the given `ProtocolId`s.
	RandomKademliaStarted(Vec<ProtocolId>),
}

impl NetworkBehaviour for DiscoveryBehaviour {
	type ProtocolsHandler = MultiHandler<ProtocolId, KademliaHandler<QueryId>>;
	type OutEvent = DiscoveryOut;

	fn new_handler(&mut self) -> Self::ProtocolsHandler {
		let iter = self.kademlias.iter_mut()
			.map(|(p, k)| (p.clone(), NetworkBehaviour::new_handler(k)));

		MultiHandler::try_from_iter(iter)
			.expect("There can be at most one handler per `ProtocolId` and \
				protocol names contain the `ProtocolId` so no two protocol \
				names in `self.kademlias` can be equal which is the only error \
				`try_from_iter` can return, therefore this call is guaranteed \
				to succeed; qed")
	}

	fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
		let mut list = self.user_defined.iter()
			.filter_map(|(p, a)| if p == peer_id { Some(a.clone()) } else { None })
			.collect::<Vec<_>>();

		{
			let mut list_to_filter = Vec::new();
			for k in self.kademlias.values_mut() {
				list_to_filter.extend(k.addresses_of_peer(peer_id))
			}

			#[cfg(not(target_os = "unknown"))]
			list_to_filter.extend(self.mdns.addresses_of_peer(peer_id));

			if !self.allow_private_ipv4 {
				list_to_filter.retain(|addr| {
					if let Some(Protocol::Ip4(addr)) = addr.iter().next() {
						if addr.is_private() {
							return false;
						}
					}

					true
				});
			}

			list.extend(list_to_filter);
		}

		trace!(target: "sub-libp2p", "Addresses of {:?}: {:?}", peer_id, list);

		list
	}

	fn inject_connection_established(&mut self, peer_id: &PeerId, conn: &ConnectionId, endpoint: &ConnectedPoint) {
		self.num_connections += 1;
		for k in self.kademlias.values_mut() {
			NetworkBehaviour::inject_connection_established(k, peer_id, conn, endpoint)
		}
	}

	fn inject_connected(&mut self, peer_id: &PeerId) {
		for k in self.kademlias.values_mut() {
			NetworkBehaviour::inject_connected(k, peer_id)
		}
	}

	fn inject_connection_closed(&mut self, peer_id: &PeerId, conn: &ConnectionId, endpoint: &ConnectedPoint) {
		self.num_connections -= 1;
		for k in self.kademlias.values_mut() {
			NetworkBehaviour::inject_connection_closed(k, peer_id, conn, endpoint)
		}
	}

	fn inject_disconnected(&mut self, peer_id: &PeerId) {
		for k in self.kademlias.values_mut() {
			NetworkBehaviour::inject_disconnected(k, peer_id)
		}
	}

	fn inject_addr_reach_failure(
		&mut self,
		peer_id: Option<&PeerId>,
		addr: &Multiaddr,
		error: &dyn std::error::Error
	) {
		for k in self.kademlias.values_mut() {
			NetworkBehaviour::inject_addr_reach_failure(k, peer_id, addr, error)
		}
	}

	fn inject_event(
		&mut self,
		peer_id: PeerId,
		connection: ConnectionId,
		(pid, event): <Self::ProtocolsHandler as ProtocolsHandler>::OutEvent,
	) {
		if let Some(kad) = self.kademlias.get_mut(&pid) {
			return kad.inject_event(peer_id, connection, event)
		}
		log::error!(target: "sub-libp2p",
			"inject_node_event: no kademlia instance registered for protocol {:?}",
			pid)
	}

	fn inject_new_external_addr(&mut self, addr: &Multiaddr) {
		let new_addr = addr.clone()
			.with(Protocol::P2p(self.local_peer_id.clone().into()));
		info!(target: "sub-libp2p", "???? Discovered new external address for our node: {}", new_addr);
		for k in self.kademlias.values_mut() {
			NetworkBehaviour::inject_new_external_addr(k, addr)
		}
	}

	fn inject_expired_listen_addr(&mut self, addr: &Multiaddr) {
		for k in self.kademlias.values_mut() {
			NetworkBehaviour::inject_expired_listen_addr(k, addr)
		}
	}

	fn inject_dial_failure(&mut self, peer_id: &PeerId) {
		for k in self.kademlias.values_mut() {
			NetworkBehaviour::inject_dial_failure(k, peer_id)
		}
	}

	fn inject_new_listen_addr(&mut self, addr: &Multiaddr) {
		for k in self.kademlias.values_mut() {
			NetworkBehaviour::inject_new_listen_addr(k, addr)
		}
	}

	fn inject_listener_error(&mut self, id: ListenerId, err: &(dyn std::error::Error + 'static)) {
		for k in self.kademlias.values_mut() {
			NetworkBehaviour::inject_listener_error(k, id, err)
		}
	}

	fn inject_listener_closed(&mut self, id: ListenerId, reason: Result<(), &io::Error>) {
		for k in self.kademlias.values_mut() {
			NetworkBehaviour::inject_listener_closed(k, id, reason)
		}
	}

	fn poll(
		&mut self,
		cx: &mut Context,
		params: &mut impl PollParameters,
	) -> Poll<
		NetworkBehaviourAction<
			<Self::ProtocolsHandler as ProtocolsHandler>::InEvent,
			Self::OutEvent,
		>,
	> {
		// Immediately process the content of `discovered`.
		if let Some(ev) = self.pending_events.pop_front() {
			return Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev));
		}

		// Poll the stream that fires when we need to start a random Kademlia query.
		while let Poll::Ready(_) = self.next_kad_random_query.poll_unpin(cx) {
			let actually_started = if self.num_connections < self.discovery_only_if_under_num {
				let random_peer_id = PeerId::random();
				debug!(target: "sub-libp2p",
					"Libp2p <= Starting random Kademlia request for {:?}",
					random_peer_id);
				for k in self.kademlias.values_mut() {
					k.get_closest_peers(random_peer_id.clone());
				}
				true
			} else {
				debug!(
					target: "sub-libp2p",
					"Kademlia paused due to high number of connections ({})",
					self.num_connections
				);
				false
			};

			// Schedule the next random query with exponentially increasing delay,
			// capped at 60 seconds.
			self.next_kad_random_query = Delay::new(self.duration_to_next_kad);
			self.duration_to_next_kad = cmp::min(self.duration_to_next_kad * 2,
				Duration::from_secs(60));

			if actually_started {
				let ev = DiscoveryOut::RandomKademliaStarted(self.kademlias.keys().cloned().collect());
				return Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev));
			}
		}

		// Poll Kademlias.
		for (pid, kademlia) in &mut self.kademlias {
			while let Poll::Ready(ev) = kademlia.poll(cx, params) {
				match ev {
					NetworkBehaviourAction::GenerateEvent(ev) => match ev {
						KademliaEvent::RoutingUpdated { peer, .. } => {
							let ev = DiscoveryOut::Discovered(peer);
							return Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev));
						}
						KademliaEvent::UnroutablePeer { peer, .. } => {
							let ev = DiscoveryOut::UnroutablePeer(peer);
							return Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev));
						}
						KademliaEvent::RoutablePeer { .. } | KademliaEvent::PendingRoutablePeer { .. } => {
							// We are not interested in these events at the moment.
						}
						KademliaEvent::QueryResult { result: QueryResult::GetClosestPeers(res), .. } => {
							match res {
								Err(GetClosestPeersError::Timeout { key, peers }) => {
									debug!(target: "sub-libp2p",
										"Libp2p => Query for {:?} timed out with {} results",
										HexDisplay::from(&key), peers.len());
								},
								Ok(ok) => {
									trace!(target: "sub-libp2p",
										"Libp2p => Query for {:?} yielded {:?} results",
										HexDisplay::from(&ok.key), ok.peers.len());
									if ok.peers.is_empty() && self.num_connections != 0 {
										debug!(target: "sub-libp2p", "Libp2p => Random Kademlia query has yielded empty \
											results");
									}
								}
							}
						}
						KademliaEvent::QueryResult { result: QueryResult::GetRecord(res), .. } => {
							let ev = match res {
								Ok(ok) => {
									let results = ok.records
										.into_iter()
										.map(|r| (r.record.key, r.record.value))
										.collect();

									DiscoveryOut::ValueFound(results)
								}
								Err(e @ libp2p::kad::GetRecordError::NotFound { .. }) => {
									trace!(target: "sub-libp2p",
										"Libp2p => Failed to get record: {:?}", e);
									DiscoveryOut::ValueNotFound(e.into_key())
								}
								Err(e) => {
									warn!(target: "sub-libp2p",
										"Libp2p => Failed to get record: {:?}", e);
									DiscoveryOut::ValueNotFound(e.into_key())
								}
							};
							return Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev));
						}
						KademliaEvent::QueryResult { result: QueryResult::PutRecord(res), .. } => {
							let ev = match res {
								Ok(ok) => DiscoveryOut::ValuePut(ok.key),
								Err(e) => {
									warn!(target: "sub-libp2p",
										"Libp2p => Failed to put record: {:?}", e);
									DiscoveryOut::ValuePutFailed(e.into_key())
								}
							};
							return Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev));
						}
						KademliaEvent::QueryResult { result: QueryResult::RepublishRecord(res), .. } => {
							match res {
								Ok(ok) => debug!(target: "sub-libp2p",
									"Libp2p => Record republished: {:?}",
									ok.key),
								Err(e) => warn!(target: "sub-libp2p",
									"Libp2p => Republishing of record {:?} failed with: {:?}",
									e.key(), e)
							}
						}
						// We never start any other type of query.
						e => {
							warn!(target: "sub-libp2p", "Libp2p => Unhandled Kademlia event: {:?}", e)
						}
					}
					NetworkBehaviourAction::DialAddress { address } =>
						return Poll::Ready(NetworkBehaviourAction::DialAddress { address }),
					NetworkBehaviourAction::DialPeer { peer_id, condition } =>
						return Poll::Ready(NetworkBehaviourAction::DialPeer { peer_id, condition }),
					NetworkBehaviourAction::NotifyHandler { peer_id, handler, event } =>
						return Poll::Ready(NetworkBehaviourAction::NotifyHandler {
							peer_id,
							handler,
							event: (pid.clone(), event)
						}),
					NetworkBehaviourAction::ReportObservedAddr { address } =>
						return Poll::Ready(NetworkBehaviourAction::ReportObservedAddr { address }),
				}
			}
		}

		// Poll mDNS.
		#[cfg(not(target_os = "unknown"))]
		while let Poll::Ready(ev) = self.mdns.poll(cx, params) {
			match ev {
				NetworkBehaviourAction::GenerateEvent(event) => {
					match event {
						MdnsEvent::Discovered(list) => {
							if self.num_connections >= self.discovery_only_if_under_num {
								continue;
							}

							self.pending_events.extend(list.map(|(peer_id, _)| DiscoveryOut::Discovered(peer_id)));
							if let Some(ev) = self.pending_events.pop_front() {
								return Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev));
							}
						},
						MdnsEvent::Expired(_) => {}
					}
				},
				NetworkBehaviourAction::DialAddress { address } =>
					return Poll::Ready(NetworkBehaviourAction::DialAddress { address }),
				NetworkBehaviourAction::DialPeer { peer_id, condition } =>
					return Poll::Ready(NetworkBehaviourAction::DialPeer { peer_id, condition }),
				NetworkBehaviourAction::NotifyHandler { event, .. } =>
					match event {},		// `event` is an enum with no variant
				NetworkBehaviourAction::ReportObservedAddr { address } =>
					return Poll::Ready(NetworkBehaviourAction::ReportObservedAddr { address }),
			}
		}

		Poll::Pending
	}
}

#[cfg(test)]
mod tests {
	use crate::config::ProtocolId;
	use futures::prelude::*;
	use libp2p::identity::Keypair;
	use libp2p::Multiaddr;
	use libp2p::core::upgrade;
	use libp2p::core::transport::{Transport, MemoryTransport};
	use libp2p::core::upgrade::{InboundUpgradeExt, OutboundUpgradeExt};
	use libp2p::swarm::Swarm;
	use std::{collections::HashSet, task::Poll};
	use super::{DiscoveryConfig, DiscoveryOut};

	#[test]
	fn discovery_working() {
		let mut user_defined = Vec::new();

		// Build swarms whose behaviour is `DiscoveryBehaviour`.
		let mut swarms = (0..25).map(|_| {
			let keypair = Keypair::generate_ed25519();
			let keypair2 = keypair.clone();

			let transport = MemoryTransport
				.and_then(move |out, endpoint| {
					let secio = libp2p::secio::SecioConfig::new(keypair2);
					libp2p::core::upgrade::apply(
						out,
						secio,
						endpoint,
						upgrade::Version::V1
					)
				})
				.and_then(move |(peer_id, stream), endpoint| {
					let peer_id2 = peer_id.clone();
					let upgrade = libp2p::yamux::Config::default()
						.map_inbound(move |muxer| (peer_id, muxer))
						.map_outbound(move |muxer| (peer_id2, muxer));
					upgrade::apply(stream, upgrade, endpoint, upgrade::Version::V1)
				});

			let behaviour = {
				let protocol_id: &[u8] = b"/test/kad/1.0.0";

				let mut config = DiscoveryConfig::new(keypair.public());
				config.with_user_defined(user_defined.clone())
					.allow_private_ipv4(true)
					.allow_non_globals_in_dht(true)
					.discovery_limit(50)
					.add_protocol(ProtocolId::from(protocol_id));

				config.finish()
			};

			let mut swarm = Swarm::new(transport, behaviour, keypair.public().into_peer_id());
			let listen_addr: Multiaddr = format!("/memory/{}", rand::random::<u64>()).parse().unwrap();

			if user_defined.is_empty() {
				user_defined.push((keypair.public().into_peer_id(), listen_addr.clone()));
			}

			Swarm::listen_on(&mut swarm, listen_addr.clone()).unwrap();
			(swarm, listen_addr)
		}).collect::<Vec<_>>();

		// Build a `Vec<HashSet<PeerId>>` with the list of nodes remaining to be discovered.
		let mut to_discover = (0..swarms.len()).map(|n| {
			(0..swarms.len()).filter(|p| *p != n)
				.map(|p| Swarm::local_peer_id(&swarms[p].0).clone())
				.collect::<HashSet<_>>()
		}).collect::<Vec<_>>();

		let fut = futures::future::poll_fn(move |cx| {
			'polling: loop {
				for swarm_n in 0..swarms.len() {
					match swarms[swarm_n].0.poll_next_unpin(cx) {
						Poll::Ready(Some(e)) => {
							match e {
								DiscoveryOut::UnroutablePeer(other) => {
									// Call `add_self_reported_address` to simulate identify happening.
									let addr = swarms.iter().find_map(|(s, a)|
										if s.local_peer_id == other {
											Some(a.clone())
										} else {
											None
										})
										.unwrap();
									swarms[swarm_n].0.add_self_reported_address(&other, addr);
								},
								DiscoveryOut::Discovered(other) => {
									to_discover[swarm_n].remove(&other);
								}
								_ => {}
							}
							continue 'polling
						}
						_ => {}
					}
				}
				break
			}

			if to_discover.iter().all(|l| l.is_empty()) {
				Poll::Ready(())
			} else {
				Poll::Pending
			}
		});

		futures::executor::block_on(fut);
	}
}
