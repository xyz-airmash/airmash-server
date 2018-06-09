//! This module contains a system to

use shrev::*;
use specs::*;
use specs::world::EntitiesRes;

use fnv::FnvHashMap;
use types::*;

use systems::collision::array2d::Array2D;
use systems::collision::bucket::*;
use systems::collision::terrain::Terrain;

// Buckets are configurable here
pub const BUCKETS_Y: usize = 64;
pub const BUCKETS_X: usize = BUCKETS_Y * 2;
pub const BUCKET_WIDTH: f32 = (32768.0 / ((BUCKETS_Y * 2) as f64)) as f32;
pub const BUCKET_HEIGHT: f32 = (32768.0 / (BUCKETS_Y as f64)) as f32;

#[derive(Default)]
pub struct CollisionSystem {
	terrain: Terrain
}

#[derive(SystemData)]
pub struct CollisionSystemData<'a> {
	pub entities: Entities<'a>,
	pub collisions: Write<'a, EventChannel<Collision>>,
	pub config: Read<'a, Config>,
	pub pos: ReadStorage<'a, Position>,
	pub rot: ReadStorage<'a, Rotation>,
	pub planes: ReadStorage<'a, Plane>,
	pub teams: ReadStorage<'a, Team>,
}

impl CollisionSystem {
	pub fn new() -> Self {
		Self::default()
	}
}

/// TODO: Replace this with something that doesn't
/// need to allocate (a generator most likely).
/// Note: generators are still a nightly-only feature
pub fn intersected_buckets(pos: Position, rad: Distance) -> impl Iterator<Item = (usize, usize)> {
	let mut vals = vec![];

	const ADJUST_Y: f32 = (BUCKETS_Y / 2) as f32 + 0.5;
	const ADJUST_X: f32 = (BUCKETS_X / 2) as f32 + 0.5;

	let y_max =
		(((pos.y + rad).inner() / BUCKET_HEIGHT).ceil()  + ADJUST_Y) as isize;
	let y_min =
		(((pos.y - rad).inner() / BUCKET_HEIGHT).floor() + ADJUST_Y) as isize;
	let x_max =
		(((pos.x + rad).inner() / BUCKET_WIDTH).ceil()   + ADJUST_X) as isize;
	let x_min =
		(((pos.x - rad).inner() / BUCKET_WIDTH).floor()  + ADJUST_X) as isize;

	trace!(target: "server", "Checking HC ({:?}, {})", pos, rad);
	trace!(target: "server", "HC BB {} {} {} {}", y_max, y_min, x_max, x_min);
	

	for x in x_min.max(0)..x_max.min(BUCKETS_X as isize) {
		for y in y_min.max(0)..y_max.min(BUCKETS_Y as isize) {
			vals.push((x as usize, y as usize));
		}
	}

	vals.into_iter()
}

impl<'a> System<'a> for CollisionSystem {
	type SystemData = CollisionSystemData<'a>;

	fn setup(&mut self, res: &mut Resources) {
		Self::SystemData::setup(res);

		self.terrain = Terrain::from_default(
			&*res.fetch::<EntitiesRes>()
		);

		// Hopefully 1000 collision events is enough during
		// each 16ms frame. If not, this number should be
		// increased.
		res.insert::<EventChannel<Collision>>(EventChannel::with_capacity(1000));
	}

	fn run(&mut self, mut data: Self::SystemData) {
		let mut buckets = self.terrain.buckets.clone();

		(
			&*data.entities,
			&data.pos,
			&data.rot,
			&data.planes,
			&data.teams,
		).join()
			.for_each(|(ent, pos, rot, plane, team)| {
				let ref cfg = data.config.planes[*plane];

				for hc in cfg.hit_circles.iter() {
					let offset = hc.offset.rotate(*rot);

					let circle = HitCircle {
						pos: *pos + offset,
						rad: hc.radius,
						layer: team.0,
						ent: ent,
					};

					for coord in intersected_buckets(*pos + offset, hc.radius) {
						trace!(target: "server", "Added to bucket {:?}", coord);
						buckets[coord].push(circle);
					}
				}
			});

		// TODO: Parallelize this (figure out par_iter with aggregation)
		let mut isects = vec![];
		buckets.iter().for_each(|bucket| {
			bucket.collide(&mut isects);
		});

		data.collisions.iter_write(isects.into_iter());
	}
}