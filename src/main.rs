use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
    window::CursorMoved,
    sprite::collide_aabb::{collide, Collision}
};

use std::collections::HashMap;

mod systems;
use crate::systems::*;

pub struct AudioLib {
    audio_files: HashMap<String, Handle<AudioSource>>
}
impl Default for AudioLib {
    fn default() -> AudioLib {
        AudioLib {
            audio_files: HashMap::new()
        }
    }
}

#[derive(Default)]
pub struct InputTracking {
    pub last_mouse_pos: Vec2
}

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.9)))
        .insert_resource(InputTracking::default())
        .insert_resource(GameState::default())
        .insert_resource(AudioLib::default())
        .insert_resource(WindowDescriptor{
            title: "Go".to_string(),
            width: 320.,
            height: 320.,
            vsync: true,
            ..Default::default()

        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(animate_sprite_system.system())
        .add_system(input.system())
        .run();
}

fn input(   
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut input_tracking: ResMut<InputTracking>,
    mut game_state: ResMut<GameState>,
    mut collider_query: Query<(&mut TextureAtlasSprite, &Transform, &mut TileSpriteData)>,
    audio: Res<Audio>,
    audio_lib: Res<AudioLib>
) {

    for event in cursor_moved_events.iter() {
        input_tracking.last_mouse_pos = event.position;
    }

    for event in mouse_button_input_events.iter() {
        match event.button {
            MouseButton::Left => {
                //Ok issue some shit yo
                for (sprite, transform, mut tile) in collider_query.iter_mut() {
                    match game_state.game_board[tile.index].tile_state {
                        Tile::Empty => {
                            let dif = (transform.translation - Vec3::new(input_tracking.last_mouse_pos.x, input_tracking.last_mouse_pos.y, 0.0)).length();
                            if dif < 16.0 {
                                game_state.make_move(tile.index);
                                audio.play(audio_lib.audio_files["clack"].clone());
                            }
                        },_ => {}
                    }
                }
            },_ => {}
        }
    }
}


fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    game_state: Res<GameState>,
    mut query: Query<(&mut TextureAtlasSprite, &Handle<TextureAtlas>, &TileSpriteData)>,
) {
    for (mut sprite, texture_atlas_handle, tile) in query.iter_mut() {
        let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
        match game_state.game_board[tile.index].tile_state {
            Tile::Empty => {
                sprite.index = 2;
            },
            Tile::Black => {
                sprite.index = 0;
            },
            Tile::White => {
                sprite.index = 1;
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    audio: Res<Audio>,
    mut audio_lib: ResMut<AudioLib>
) {
    let texture_handle = asset_server.load("textures/go/go_spritesheet.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 1, 3);
    let mut texture_atlas_handle = texture_atlases.add(texture_atlas);
    let mut ocb = OrthographicCameraBundle::new_2d();
    ocb.transform.translation = Vec3::new(160.0, 160., 0.);
    commands.spawn_bundle(ocb);
    for i in 0..81 {
        let x = i % 9;
        let y = i / 9;
        let transform = Transform::from_translation(Vec3::new(x as f32 * 32.0 + 32., y as f32 * 32.0 + 32., 0.0));
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                transform,
                ..Default::default()
            })
            .insert(TileSpriteData::new(i));
    }

    let music = asset_server.load("sounds/clack.mp3");
    audio_lib.audio_files.insert("clack".to_string(), music);
}
