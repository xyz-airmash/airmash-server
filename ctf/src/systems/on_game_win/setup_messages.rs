
use specs::*;

use server::*;
use server::component::event::TimerEvent;
use server::consts::timer::DELAYED_MESSAGE;
use server::types::FutureDispatcher;
use server::protocol::ServerMessageType;
use server::protocol::server::ServerMessage;

use std::time::Duration;
use component::*;
use systems::on_flag::CheckWin;

const MESSAGE_1_MIN: &'static str = "New game starting in 1 minute";
const MESSAGE_30_SECONDS: &'static str = "Game starting in 30 seconds - shuffling teams";
const MESSAGE_10_SECONDS: &'static str = "Game starting in 10 seconds";
const MESSAGE_5_SECONDS: &'static str = "Game starting in 5 seconds";
const MESSAGE_4_SECONDS: &'static str = "Game starting in 4 seconds";
const MESSAGE_3_SECONDS: &'static str = "Game starting in 3 seconds";
const MESSAGE_2_SECONDS: &'static str = "Game starting in 2 seconds";
const MESSAGE_1_SECONDS: &'static str = "Game starting in a second";
const MESSAGE_0_SECONDS: &'static str = "Game starting!";

const MESSAGES: [(u64, u32, &'static str); 9] = [
	(12, 25, MESSAGE_1_MIN),
	(7,  55, MESSAGE_30_SECONDS),
	(7,  75, MESSAGE_10_SECONDS),
	(2,  80, MESSAGE_5_SECONDS),
	(2,  81, MESSAGE_4_SECONDS),
	(2,  82, MESSAGE_3_SECONDS),
	(2,  83, MESSAGE_2_SECONDS),
	(2,  84, MESSAGE_1_SECONDS),
	(3,  85, MESSAGE_0_SECONDS)
];

#[derive(Default)]
pub struct SetupMessages {
	reader: Option<OnGameWinReader>
}

#[derive(SystemData)]
pub struct SetupMessagesData<'a> {
	channel: Read<'a, OnGameWin>,
	future: ReadExpect<'a, FutureDispatcher>,
}

impl<'a> System<'a> for SetupMessages {
	type SystemData = SetupMessagesData<'a>;

	fn setup(&mut self, res: &mut Resources) {
		Self::SystemData::setup(res);

		self.reader = Some(
			res.fetch_mut::<OnGameWin>().register_reader()
		);
	}

	fn run(&mut self, data: Self::SystemData) {
		for _ in data.channel.read(self.reader.as_mut().unwrap()) {
			for (delay, duration, msg) in MESSAGES.iter() {
				data.future.run_delayed(
					Duration::from_secs(*delay),
					move |inst| Some(TimerEvent {
						ty: *DELAYED_MESSAGE,
						instant: inst,
						data: Some(Box::new(ServerMessage {
							ty: ServerMessageType::TimeToGameStart,
							duration: *duration * 1000,
							text: msg.to_string()
						}))
					})
				);
			}
		}
	}
}

impl SystemInfo for SetupMessages {
	type Dependencies = CheckWin;

	fn name() -> &'static str {
		concat!(module_path!(), "::", line!())
	}

	fn new() -> Self {
		Self::default()
	}
}