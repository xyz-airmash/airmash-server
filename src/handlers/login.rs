
use specs::*;
use uuid::Uuid;
use shrev::{ReaderId, EventChannel};
use websocket::OwnedMessage;
use airmash_protocol::client::Login;
use airmash_protocol::server::{ServerPacket, PlayerNew, PlayerLevel};
use airmash_protocol::{
	client,
	server,
	to_bytes,
	KeyState as ProtocolKeyState,
	PlayerStatus,
	PlaneType,
	Upgrades as ProtocolUpgrades,
	FlagCode,
	PlayerLevelType
};

use std::str::FromStr;

use types::*;

// Login needs write access to just 
// about everything
#[derive(SystemData)]
pub struct LoginSystemData<'a> {
	pub entities: Entities<'a>,
	pub position: WriteStorage<'a, Position>,
	pub speed:    WriteStorage<'a, Speed>,
	pub accel:    WriteStorage<'a, Accel>,
	pub energy:   WriteStorage<'a, Energy>,
	pub health:   WriteStorage<'a, Health>,
	pub rot:      WriteStorage<'a, Rotation>,
	pub keystate: WriteStorage<'a, KeyState>,
	pub name:     WriteStorage<'a, Name>,
	pub session:  WriteStorage<'a, Session>,
	pub powerups: WriteStorage<'a, Powerups>,
	pub upgrades: WriteStorage<'a, Upgrades>,
	pub score:    WriteStorage<'a, Score>,
	pub level:    WriteStorage<'a, Level>,
	pub team:     WriteStorage<'a, Team>,
	pub flag:     WriteStorage<'a, Flag>,
	pub plane:    WriteStorage<'a, Plane>,
	pub status:   WriteStorage<'a, Status>,
	pub conns:    Write<'a, Connections>,
	pub associated_conn: WriteStorage<'a, AssociatedConnection>
}

pub struct LoginHandler {
	reader: Option<ReaderId<(ConnectionId, Login)>>
}

impl LoginHandler {
	pub fn new() -> Self {
		Self {
			reader: None
		}
	}

	fn send_new<'a>(
		data: &LoginSystemData<'a>,
		entity: u32,
		login: &Login
	) {
		let player_new = PlayerNew {
			id:     entity as u16,
			status: PlayerStatus::Alive,
			name:   login.name.clone(),
			ty:     PlaneType::Predator,
			team:   0,
			pos_x:  0.0,
			pos_y:  0.0,
			rot:    0.0,
			flag:   FlagCode::UnitedNations,
			upgrades: ProtocolUpgrades(0)
		};

		data.conns.send_to_all(OwnedMessage::Binary(
			to_bytes(&ServerPacket::PlayerNew(player_new)).unwrap()
		));
	}

	fn send_level<'a>(
		data: &LoginSystemData<'a>,
		entity: u32,
		_login: &Login
	) {
		let player_level = PlayerLevel {
			id:     entity as u16,
			ty:     PlayerLevelType::Login,
			level:  0
		};

		data.conns.send_to_all(OwnedMessage::Binary(
			to_bytes(&ServerPacket::PlayerLevel(player_level)).unwrap()
		));
	}

	fn get_player_data<'a>(
		data: &LoginSystemData<'a>
	) -> Vec<server::LoginPlayer> {
		// This formatting is ugly :(
		// The size of the join makes it necessary

		(
			&*data.entities,
			&data.position,
			&data.rot,
			&data.plane,
			&data.name,
			&data.flag,
			&data.upgrades,
			&data.level,
			&data.status,
			&data.team,
			&data.powerups
		).join()
		.map({
			|(ent, pos, rot, plane, name, flag, 
				upgrades, level, status, team, powerups)| 
			{
				let mut upgrade_field = ProtocolUpgrades(0);
				upgrade_field.set_speed(upgrades.speed);
				upgrade_field.set(ProtocolUpgrades::INFERNO, powerups.inferno);
				upgrade_field.set(ProtocolUpgrades::SHIELD, powerups.shield);

				server::LoginPlayer {
					id:     ent.id() as u16,
					status: status.0,
					level:  level.0,
					name:   name.0.clone(),
					ty:     plane.0,
					team:   team.0,
					pos_x:  pos.x.inner(),
					pos_y:  pos.y.inner(),
					rot:    rot.inner(),
					flag:   flag.0,
					upgrades: upgrade_field
				}
			}
		}).collect()
	}

	fn do_login<'a>(
		data: &mut LoginSystemData<'a>,
		conn: ConnectionId,
		login: Login
	) {
		let entity = data.entities.create();

		if entity.id() > 0xFFFF {
			error!(
				target: "server",
				"Entity created with id greater than 0xFFFF. Aborting to avoid sending invalid packets."
			);
			panic!("Entity created with invalid id.");
		}

		info!(
			target: "server",
			"{:?} logging on as {} with id {}",
			conn, login.name, entity.id()
		);

		Self::send_new(data, entity.id(), &login);
		Self::send_level(data, entity.id(), &login);

		let session = match Uuid::from_str(&login.session) {
			Ok(s) => Some(s),
			Err(_) => None
		};

		let resp = server::Login {
			clock: 0,
			id:    entity.id() as u16,
			room:  "test".to_string(),
			success: true,
			token: login.session,
			team:  0,
			ty:    PlaneType::Predator,
			players: Self::get_player_data(data)
		};

		data.conns.associate(
			conn, 
			entity,
			ConnectionType::Primary
		);

		data.conns.send_to(conn, OwnedMessage::Binary(
			to_bytes(&ServerPacket::Login(resp)).unwrap()
		));

		// Set all possible pieces of state for a plane
		data.position.insert(entity, Position::default()).unwrap();
		data.speed.insert(entity, Speed::default()).unwrap();
		data.accel.insert(entity, Accel::default()).unwrap();
		data.energy.insert(entity, Energy::new(1.0)).unwrap();
		data.health.insert(entity, Health::new(1.0)).unwrap();
		data.rot.insert(entity, Rotation::default()).unwrap();
		data.keystate.insert(entity, KeyState::default()).unwrap();
		data.name.insert(entity, Name(login.name)).unwrap();
		data.session.insert(entity, Session(session)).unwrap();
		data.powerups.insert(entity, Powerups::default()).unwrap();
		data.upgrades.insert(entity, Upgrades::default()).unwrap();
		data.score.insert(entity, Score(0)).unwrap();
		data.level.insert(entity, Level(0)).unwrap();
		data.team.insert(entity, Team(0)).unwrap();
		data.flag.insert(entity, 
			Flag(FlagCode::from_str(&login.flag).unwrap_or(FlagCode::UnitedNations))
		).unwrap();
		data.plane.insert(entity, Plane::default()).unwrap();
		data.status.insert(entity, Status::default()).unwrap();
		data.associated_conn.insert(entity, AssociatedConnection(conn)).unwrap();
	}
}

impl<'a> System<'a> for LoginHandler {
	type SystemData = (
		Read<'a, EventChannel<(ConnectionId, Login)>>, 
		LoginSystemData<'a>
	);

	fn setup(&mut self, res: &mut Resources) {
		self.reader = Some(
			res.fetch_mut::<EventChannel<(ConnectionId, Login)>>().register_reader()
		);

		Self::SystemData::setup(res);
	}

	fn run(&mut self, (channel, mut data): Self::SystemData) {
		if let Some(ref mut reader) = self.reader {
			for evt in channel.read(reader).cloned() { 
				Self::do_login(&mut data, evt.0, evt.1);
			}
		}
	}
}
