mod send_player_fire;
mod set_last_shot;

pub use self::send_player_fire::SendPlayerFire;
pub use self::set_last_shot::SetLastShot;

pub type AllFireHandlers = (SendPlayerFire, SetLastShot);

use systems;

pub type KnownEventSources = (systems::missile::MissileFireHandler);
