use fnv::FnvHashSet;
use specs::prelude::*;

use types::collision::*;
use types::*;

use component::channel::*;
use component::collision::PlaneGrid;
use component::event::PlayerMissileCollision;
use component::flag::*;

pub struct PlayerMissileCollisionSystem;

#[derive(SystemData)]
pub struct PlayerMissileCollisionSystemData<'a> {
	pub channel: Write<'a, OnPlayerMissileCollision>,
	pub ent: Entities<'a>,
	pub grid: Read<'a, PlaneGrid>,

	pub mob: ReadStorage<'a, Mob>,
	pub missile_flag: ReadStorage<'a, IsMissile>,
	pub pos: ReadStorage<'a, Position>,
	pub team: ReadStorage<'a, Team>,
}

impl PlayerMissileCollisionSystem {
	pub fn new() -> Self {
		Self {}
	}
}

impl<'a> System<'a> for PlayerMissileCollisionSystem {
	type SystemData = PlayerMissileCollisionSystemData<'a>;

	fn run(&mut self, data: Self::SystemData) {
		let Self::SystemData {
			mut channel,
			ent,

			grid,
			pos,
			team,

			mob,
			missile_flag,
		} = data;

		let grid = &grid.0;

		let collisions = (&*ent, &pos, &team, &mob, &missile_flag)
			.par_join()
			.map(|(ent, &pos, &team, &mob, _)| {
				let it = COLLIDERS[&mob].iter().map(move |(offset, rad)| HitCircle {
					pos: pos + *offset,
					rad: *rad,
					layer: team.0,
					ent: ent,
				});

				grid.collide(it)
			})
			.flatten()
			.map(|x| PlayerMissileCollision(x))
			.collect::<FnvHashSet<PlayerMissileCollision>>();

		channel.iter_write(collisions.into_iter());
	}
}

use dispatch::SystemInfo;
use systems::collision::GenPlaneGrid;
use systems::PositionUpdate;

impl SystemInfo for PlayerMissileCollisionSystem {
	type Dependencies = (PositionUpdate, GenPlaneGrid);

	fn name() -> &'static str {
		concat!(module_path!(), "::", line!())
	}

	fn new() -> Self {
		Self::new()
	}
}
