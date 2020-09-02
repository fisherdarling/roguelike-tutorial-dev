use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use tcod::colors::*;
use tcod::console::*;
use tcod::input::{Key, KeyCode, KeyPressFlags, Mouse};
use tcod::map::{FovAlgorithm, Map as FovMap};

// actual size of the window
pub const SCREEN_WIDTH: i32 = 80;
pub const SCREEN_HEIGHT: i32 = 50;
pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 45;

pub const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum

pub type TcodKey = Key;
pub type TcodMouse = Mouse;

pub struct Tcod {
    pub root: Root,
    pub con: Offscreen,
    pub fov: FovMap,
}

unsafe impl Send for Tcod {}
unsafe impl Sync for Tcod {}

pub struct TcodPlugin;

impl Plugin for TcodPlugin {
    fn build(&self, app: &mut AppBuilder) {
        tcod::system::set_fps(LIMIT_FPS);

        let root = Root::initializer()
            .font("arial10x10.png", FontLayout::Tcod)
            .font_type(FontType::Greyscale)
            .size(SCREEN_WIDTH, SCREEN_HEIGHT)
            .title("Rust/libtcod tutorial")
            .init();

        let mut tcod = Tcod {
            root,
            con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
            fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
        };

        app.add_resource(tcod)
            .init_resource::<Input<(KeyCode, char)>>()
            .add_system_to_stage(stage::EVENT_UPDATE, update_input.system());
    }
}

fn update_input(mut key_input: ResMut<Input<(KeyCode, char)>>) {
    use tcod::input;
    key_input.update();

    while let Some((_, event)) = input::check_for_event(input::KEY | input::MOUSE) {
        match event {
            input::Event::Key(key_state) => {
                let e = (key_state.code, key_state.printable);
                if key_state.pressed {
                    key_input.press(e);
                } else {
                    key_input.release(e);
                }

                println!("Key: {:?}", e);
            }
            input::Event::Mouse(mouse_state) => {
                // mouse_input.send(mouse_state);
            }
        }
    }
}
