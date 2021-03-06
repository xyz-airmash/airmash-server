use specs::*;

use types::systemdata::*;
use types::*;

use dispatch::SystemInfo;

use component::channel::*;
use component::event::*;

use utils::{EventHandler, EventHandlerTypeProvider};

use protocol::server::{PlayerFire, PlayerFireProjectile};

#[derive(Default)]
pub struct SendPlayerFire;

#[derive(SystemData)]
pub struct SendPlayerFireData<'a> {
	pub entities: Entities<'a>,
	pub channel: Read<'a, OnMissileFire>,
	pub conns: Read<'a, Connections>,
	pub config: Read<'a, Config>,
	pub clock: ReadClock<'a>,

	pub mob: ReadStorage<'a, Mob>,
	pub pos: ReadStorage<'a, Position>,
	pub vel: ReadStorage<'a, Velocity>,
	pub energy: ReadStorage<'a, Energy>,
	pub energy_regen: ReadStorage<'a, EnergyRegen>,
}

impl EventHandlerTypeProvider for SendPlayerFire {
	type Event = MissileFire;
}

impl<'a> EventHandler<'a> for SendPlayerFire {
	type SystemData = SendPlayerFireData<'a>;

	fn on_event(&mut self, evt: &MissileFire, data: &mut Self::SystemData) {
		let projectiles = evt
			.missiles
			.iter()
			.filter_map(|&ent| {
				let ty = *log_none!(ent, data.mob)?;
				let info = data.config.mobs[ty].missile.unwrap();

				let vel = *log_none!(ent, data.vel)?;
				let pos = *log_none!(ent, data.pos)?;

				PlayerFireProjectile {
					id: ent.into(),
					pos: pos,
					speed: vel,
					ty: ty,
					accel: vel.normalized() * info.accel,
					max_speed: info.max_speed,
				}
				.into()
			})
			.collect::<Vec<_>>();

		let pos = *try_get!(evt.player, data.pos);

		let packet = PlayerFire {
			clock: data.clock.get(),
			id: evt.player.into(),
			energy: *try_get!(evt.player, data.energy),
			energy_regen: *try_get!(evt.player, data.energy_regen),
			projectiles,
		};

		data.conns.send_to_visible(pos, packet);
	}
}

impl SystemInfo for SendPlayerFire {
	type Dependencies = super::KnownEventSources;

	fn name() -> &'static str {
		concat!(module_path!(), "::", line!())
	}

	fn new() -> Self {
		Self::default()
	}
}
