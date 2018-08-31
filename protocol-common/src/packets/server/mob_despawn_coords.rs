use enums::MobType;
use types::{Mob, Position};

/// A missile despawned with an explosion
/// This is used when a missile
/// collides with a mountain to
/// generate an explosion client-side
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct MobDespawnCoords {
	pub id: Mob,
	#[serde(rename = "type")]
	pub ty: MobType,
	pub pos: Position,
}
