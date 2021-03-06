use specs::*;

use SystemInfo;

use systems::handlers::packet::LoginHandler;

use component::channel::*;
use component::time::*;

pub struct InitLastRepelTime {
	reader: Option<OnPlayerJoinReader>,
}

#[derive(SystemData)]
pub struct InitLastRepelTimeData<'a> {
	pub channel: Read<'a, OnPlayerJoin>,
	pub this_frame: Read<'a, ThisFrame>,

	pub join_time: WriteStorage<'a, LastRepelTime>,
}

impl<'a> System<'a> for InitLastRepelTime {
	type SystemData = InitLastRepelTimeData<'a>;

	fn setup(&mut self, res: &mut Resources) {
		Self::SystemData::setup(res);

		self.reader = Some(res.fetch_mut::<OnPlayerJoin>().register_reader());
	}

	fn run(&mut self, mut data: Self::SystemData) {
		for evt in data.channel.read(self.reader.as_mut().unwrap()) {
			data.join_time
				.insert(evt.id, LastRepelTime(data.this_frame.0))
				.unwrap();
		}
	}
}

impl SystemInfo for InitLastRepelTime {
	type Dependencies = LoginHandler;

	fn name() -> &'static str {
		concat!(module_path!(), "::", line!())
	}

	fn new() -> Self {
		Self { reader: None }
	}
}
