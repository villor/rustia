use super::network::game_listener::NewClientInfo;
use super::util::atomic_counter::AtomicU32Counter;
use super::tilemap;

pub struct NewClientRx(pub flume::Receiver<NewClientInfo>);

pub struct CreatureIdCounter(pub AtomicU32Counter);

pub struct TileMap(pub tilemap::TileMap);
