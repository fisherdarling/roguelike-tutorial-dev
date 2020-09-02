use rand::Rng;

use bevy::app::AppExit;
use bevy::prelude::*;

use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::input::KeyCode::{self, *};
use tcod::map::{FovAlgorithm, Map as FovMap};

use crate::tcod_plugin::{Tcod, SCREEN_HEIGHT, SCREEN_WIDTH};

// size of the map
pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 45;

//parameters for dungeon generator
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

pub const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
pub const FOV_LIGHT_WALLS: bool = true; // light walls or not
pub const TORCH_RADIUS: i32 = 10;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

#[derive(Debug, Default)]
struct UpdateFOV;

#[derive(Debug, Default)]
struct Player;

#[derive(Debug, Default, PartialEq, Eq)]
struct Location {
    pub x: i32,
    pub y: i32,
}

impl Location {
    fn add(&self, other: Location) -> Location {
        Location {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Debug, Default)]
struct Glyph(pub char, Color);

// A tile of the map and its properties
#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    explored: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            explored: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            explored: false,
            block_sight: true,
        }
    }
}

/// A rectangle on the map, used to characterise a room.
#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
}

impl Rect {
    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        // returns true if this rectangle intersects with another one
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}

type Map = Vec<Vec<Tile>>;

fn make_map() -> (Map, Location) {
    let mut rooms = vec![];
    let mut player_location = Location::default();
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        // random position without going out of the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);

        // run through the other rooms and see if they intersect with this one
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            // this means there are no intersections, so this room is valid

            // "paint" it to the map's tiles
            create_room(new_room, &mut map);

            // center coordinates of the new room, will be useful later
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                // this is the first room, where the player starts at
                player_location.x = new_x;
                player_location.y = new_y;
            } else {
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                // toss a coin (random bool value -- either true or false)
                if rand::random() {
                    // first move horizontally, then vertically
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    // first move vertically, then horizontally
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }

            rooms.push(new_room);
        }
    }

    (map, player_location)
}

fn create_room(room: Rect, map: &mut Map) {
    // go through the tiles in the rectangle and make them passable
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    use std::cmp;
    // horizontal tunnel. `min()` and `max()` are used in case `x1 > x2`
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    use std::cmp;
    // vertical tunnel
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

#[derive(Debug, Clone)]
struct Game {
    map: Map,
}

impl Game {
    fn passable(&self, x: i32, y: i32) -> bool {
        if let Some(tile) = self
            .map
            .get(x as usize)
            .and_then(|cols| cols.get(y as usize))
        {
            !tile.blocked
        } else {
            false
        }
    }
}

fn spawn_entities(mut commands: Commands, mut tcod: ResMut<Tcod>) {
    let (map, player_location) = make_map();
    let game = Game { map };

    // populate the FOV map, according to the generated map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }

    tcod.fov.compute_fov(
        player_location.x,
        player_location.y,
        TORCH_RADIUS,
        FOV_LIGHT_WALLS,
        FOV_ALGO,
    );

    commands
        .insert_resource(game)
        .spawn((Player, player_location, Glyph('@', Color::WHITE)));
}

fn handle_key_press(
    mut exit: ResMut<Events<AppExit>>,
    keys: Res<Input<(KeyCode, char)>>,
    game: Res<Game>,
    mut update_fov: ResMut<Events<UpdateFOV>>,
    _player: &Player,
    mut location: Mut<Location>,
) {
    if keys.pressed((Escape, '\u{1b}')) {
        exit.send(AppExit);
    }

    let mut delta = Location::default();
    if keys.pressed((Up, '\0')) {
        if game.passable(location.x, location.y - 1) {
            delta.y -= 1
        }
    }

    if keys.pressed((Down, '\0')) {
        if game.passable(location.x, location.y + 1) {
            delta.y += 1
        }
    }

    if keys.pressed((Left, '\0')) {
        if game.passable(location.x - 1, location.y) {
            delta.x -= 1
        }
    }

    if keys.pressed((Right, '\0')) {
        if game.passable(location.x + 1, location.y) {
            delta.x += 1
        }
    }

    let next = location.add(delta);
    if game.passable(next.x, next.y) && next != *location {
        *location = next;
        update_fov.send(UpdateFOV);
    }
}

fn update_fov(
    mut tcod: ResMut<Tcod>,
    update: Res<Events<UpdateFOV>>,
    _player: &Player,
    player_location: &Location,
) {
    let mut reader = update.get_reader();
    if reader.latest(&update).is_some() {
        tcod.fov.compute_fov(
            player_location.x,
            player_location.y,
            TORCH_RADIUS,
            FOV_LIGHT_WALLS,
            FOV_ALGO,
        );
    }
}

fn draw_world(
    mut tcod: ResMut<Tcod>,
    mut game: ResMut<Game>,
    mut drawables: Query<(&Location, &Glyph)>,
) {
    let Tcod { root, con, fov } = &mut *tcod;

    if !root.window_closed() {
        con.set_default_foreground(colors::WHITE);
        con.clear();

        for (location, glyph) in &mut drawables.iter() {
            con.put_char(location.x, location.y, glyph.0, BackgroundFlag::None);
        }

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let visible = fov.is_in_fov(x, y);

                let wall = game.map[x as usize][y as usize].block_sight;
                let color = match (visible, wall) {
                    // outside of field of view:
                    (false, true) => COLOR_DARK_WALL,
                    (false, false) => COLOR_DARK_GROUND,
                    // inside fov:
                    (true, true) => COLOR_LIGHT_WALL,
                    (true, false) => COLOR_LIGHT_GROUND,
                };

                let explored = &mut game.map[x as usize][y as usize].explored;

                if visible {
                    *explored = true;
                }

                if *explored {
                    // show explored tiles only (any visible tile is explored already)
                    con.set_char_background(x, y, color, BackgroundFlag::Set);
                }
            }
        }

        blit(
            con,
            (0, 0),
            (SCREEN_WIDTH, SCREEN_HEIGHT),
            root,
            (0, 0),
            1.0,
            1.0,
        );

        con.clear();
        root.flush();
    }
}

#[derive(Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(crate::tcod_plugin::TcodPlugin)
            .add_event::<UpdateFOV>()
            .add_startup_system(spawn_entities.system())
            .add_system_to_stage(stage::PRE_UPDATE, handle_key_press.system())
            .add_system_to_stage(stage::UPDATE, update_fov.system())
            .add_system_to_stage(stage::POST_UPDATE, draw_world.system());
    }
}
