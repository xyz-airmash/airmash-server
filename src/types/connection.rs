
use types::ConnectionId;

use websocket::OwnedMessage;
use websocket::client::async::Framed;
use websocket::async::{TcpStream, MessageCodec};
use futures::stream::SplitSink;
use futures::{Sink, AsyncSink};
use fnv::FnvHashMap;

use std::sync::Mutex;

pub type ConnectionSink = SplitSink<Framed<TcpStream, MessageCodec<OwnedMessage>>>;

pub struct ConnectionData {
	pub sink: Mutex<ConnectionSink>,
	pub id: ConnectionId,
	pub ty: ConnectionType,
	pub player: Option<usize>
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ConnectionType {
	Primary,
	Backup,
	Inactive
}

#[derive(Default)]
pub struct Connections(FnvHashMap<ConnectionId, ConnectionData>);

impl Connections {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add(&mut self, id: ConnectionId, sink: ConnectionSink) {
		let data = ConnectionData {
			sink:   Mutex::new(sink),
			ty:     ConnectionType::Inactive,
			player: None,
			id:     id,
		};

		self.0.insert(id, data);
	}
	pub fn remove(&mut self, id: ConnectionId) {
		self.0.remove(&id).unwrap_or_else(|| {
			error!(
				target: "server",
				"Attempted to remove non-existent connection {:?}",
				id
			);
			panic!("Nonexistent connection id {:?}", id);
		});
	}

	pub fn associate(&mut self, id: ConnectionId, player: usize, ty: ConnectionType) {
		let ref mut conn = self.0.get_mut(&id).unwrap_or_else(|| {
			error!(
				target: "server",
				"Attempted to associate non-existent connection {:?} with player {}",
				id, player
			);
			panic!("Nonexistent connection id {:?}", id);
		});

		conn.player = Some(player);
		conn.ty = ty;
	}

	fn send_sink(conn: &mut ConnectionSink, msg: OwnedMessage) {
		conn.start_send(msg).and_then(|x| {
			match x {
				AsyncSink::Ready => (),
				AsyncSink::NotReady(item) => {
					conn.poll_complete().unwrap();
					conn.start_send(item).unwrap();
				}
			}
			Ok(())
		}).unwrap();
	}

	pub fn send_to(&self, id: ConnectionId, msg: OwnedMessage) {
		let data = self.0.get(&id).unwrap_or_else(|| {
			error!(
				target: "server",
				"Tried to send to nonexistent connection {:?} this message: {:?}",
				id, msg
			);
			panic!("Nonexistent connection id {:?}", id);
		});

		Self::send_sink(&mut data.sink.lock().unwrap(), msg);
	}

	pub fn send_to_all(&self, msg: OwnedMessage) {
		self.0.values()
			.filter_map(|ref conn| {
				if conn.player.is_some() {
					if conn.ty == ConnectionType::Primary {
						return Some(&conn.sink);
					}
				}
				None
			})
			.for_each(|ref sink| {
				Self::send_sink(&mut sink.lock().unwrap(), msg.clone());
			});
	}
}