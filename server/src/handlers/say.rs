use shrev::*;
use specs::*;
use types::*;

use component::flag::*;

use protocol::client::Say;
use protocol::server::{ChatSay, Error};
use protocol::ErrorType;

pub struct SayHandler {
	reader: Option<ReaderId<(ConnectionId, Say)>>,
}

#[derive(SystemData)]
pub struct SayHandlerData<'a> {
	channel: Read<'a, EventChannel<(ConnectionId, Say)>>,
	conns: Read<'a, Connections>,

	throttled: ReadStorage<'a, IsChatThrottled>,
	muted: ReadStorage<'a, IsChatMuted>,
}

impl SayHandler {
	pub fn new() -> Self {
		Self { reader: None }
	}
}

impl<'a> System<'a> for SayHandler {
	type SystemData = SayHandlerData<'a>;

	fn setup(&mut self, res: &mut Resources) {
		self.reader = Some(
			res.fetch_mut::<EventChannel<(ConnectionId, Say)>>()
				.register_reader(),
		);

		Self::SystemData::setup(res);
	}

	fn run(&mut self, data: Self::SystemData) {
		for evt in data.channel.read(self.reader.as_mut().unwrap()) {
			let player = match data.conns.associated_player(evt.0) {
				Some(player) => player,
				None => continue,
			};

			if data.muted.get(player).is_some() {
				continue;
			}
			if data.throttled.get(player).is_some() {
				data.conns.send_to(
					evt.0,
					Error {
						error: ErrorType::ChatThrottled,
					},
				);
				continue;
			}

			let chat = ChatSay {
				id: player.into(),
				text: evt.1.text.clone(),
			};

			data.conns.send_to_all(chat);
		}
	}
}

use dispatch::SystemInfo;
use handlers::OnCloseHandler;

impl SystemInfo for SayHandler {
	type Dependencies = OnCloseHandler;

	fn new() -> Self {
		Self::new()
	}

	fn name() -> &'static str {
		concat!(module_path!(), "::", line!())
	}
}
