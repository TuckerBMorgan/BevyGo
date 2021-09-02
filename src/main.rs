use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
    window::CursorMoved,
    sprite::collide_aabb::{collide, Collision}
};

use bevy::{core::FixedTimestep, prelude::*};
use std::convert::TryInto;


use std::collections::HashMap;
use bevy_ggrs::{GGRSApp, GGRSPlugin, RollbackIdProvider};
use ggrs::{GameInput, P2PSession, P2PSpectatorSession, PlayerHandle, SyncTestSession, PlayerType};
use std::net::SocketAddr;
use structopt::StructOpt;


const INPUT_SIZE: usize = std::mem::size_of::<u32>();
const FPS: u32 = 60;

mod systems;
use crate::systems::*;


struct Player {
    handle: u32
}

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


#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    local_port: u16,
    #[structopt(short, long)]
    players: Vec<String>,
    #[structopt(short, long)]
    spectators: Vec<SocketAddr>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // read cmd line arguments
    let opt = Opt::from_args();
    let mut local_handle = 0;
    let num_players = opt.players.len();
    assert!(num_players > 0);

    //setup p2p sessions
    let mut p2p_sess = ggrs::start_p2p_session(num_players as u32, INPUT_SIZE, opt.local_port)?;
    p2p_sess.set_sparse_saving(true)?;
    for (i, player_addr) in opt.players.iter().enumerate() {
        // local player
        if player_addr == "localhost" {
            p2p_sess.add_player(PlayerType::Local, i)?;
            local_handle = i;
        } else {
            // remote players
            let remote_addr: SocketAddr = player_addr.parse()?;
            p2p_sess.add_player(PlayerType::Remote(remote_addr), i)?;
        }
    }
    
    // optionally, add spectators
    for (i, spec_addr) in opt.spectators.iter().enumerate() {
        p2p_sess.add_player(PlayerType::Spectator(*spec_addr), num_players + i)?;
    }

    p2p_sess.set_frame_delay(2, local_handle)?;
    p2p_sess.set_fps(FPS)?;
    p2p_sess.start_session()?;

    App::new()
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.9)))

        .insert_resource(WindowDescriptor{
            title: "Go".to_string(),
            width: 320.,
            height: 320.,
            vsync: true,
            ..Default::default()

        })
        .add_plugins(DefaultPlugins)
        .add_plugin(GGRSPlugin)
        
        .add_startup_system(network_setup.system())
        .with_p2p_session(p2p_sess)
        .insert_resource(InputTracking::default())
        .insert_resource(GameState::default())
        .insert_resource(AudioLib::default())
        .with_rollback_run_criteria(FixedTimestep::steps_per_second(FPS as f64))
        .with_input_system(input.system())
        .add_system(animate_sprite_system.system())
        .add_rollback_system(make_move_system.system())
        .register_rollback_type::<GameState>()
        
        .run();
    Ok(())
}


pub fn input(   
    _handle: In<PlayerHandle>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut input_tracking: ResMut<InputTracking>,
    mut game_state: ResMut<GameState>,
    mut tile_query: Query<(&Transform, &mut TileSpriteData)>
) -> Vec<u8> {
    for event in cursor_moved_events.iter() {
        input_tracking.last_mouse_pos = event.position;
    }
    let mut index = 0;
    for event in mouse_button_input_events.iter() {
        match event.button {
            MouseButton::Left => {

                let mut input_encoding : u32 = 0;

                input_encoding |= (input_tracking.last_mouse_pos.x as u32);
                input_encoding = input_encoding << 16;
                input_encoding |= (input_tracking.last_mouse_pos.y as u32);
                println!("Sending:");
                println!("{:?} {:?}", input_tracking.last_mouse_pos.x, input_tracking.last_mouse_pos.y);
                return input_encoding.to_be_bytes().to_vec();
       
            },_ => {}
        }
    }

    vec![0, 0, 0, 0]
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

fn make_move_system(
    mut game_state: ResMut<GameState>,
    mut tile_query: Query<(&Transform, &mut TileSpriteData)>,
    inputs: Res<Vec<GameInput>>
) {

    
    for i in inputs.iter() {
        let mut sum : u32  = 0;
        for x in 0..4 {
            sum = sum << 8;
            sum |= i.buffer[x] as u32;

        }
        if sum > 0 {
            //WE HAVE A MESSAGE FROM THE OTHER SIDE
            let x = sum >> 16;
            sum = sum << 16;
            let y = sum >> 16;
            println!("I GOt");
            println!("{} {}", x, y);


            for (transform, mut tile) in tile_query.iter_mut() {
    
                match game_state.game_board[tile.index].tile_state {
                    Tile::Empty => {
    
                        let dif = (transform.translation - Vec3::new(x as f32, y as f32, 0.0)).length();
    
                        if dif < 16.0 {
                            game_state.game_board[tile.index].tile_state = game_state.current_player;
                            if game_state.current_player_id == 0 {
                                game_state.current_player_id = 1;
                                game_state.current_player = Tile::Black;
                            }
                            else {
                                game_state.current_player_id = 0;
                                game_state.current_player = Tile::White;
                            }
                        }
                    },_ => {}
                }
            }
            
        }
    }
}



fn network_setup(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    rip: ResMut<RollbackIdProvider>,
    mut audio_lib: ResMut<AudioLib>,
    asset_server: Res<AssetServer>,
    p2p_session: Option<Res<P2PSession>>,
    audio: Res<Audio>,
    session: Option<Res<SyncTestSession>>
) {
    let texture_handle = asset_server.load("textures/go/go_spritesheet.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 1, 3);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
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
    /*
    let num_players = p2p_session.map(|s|s.num_players()).expect("No session");
    let transform = Transform::from_scale(Vec3::new(0.0, 0.0, 0.0));
    for i in 0..num_players {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                transform,
                ..Default::default()
            })
            .insert(Player{handle: i});
    }
 */
    let music = asset_server.load("sounds/clack.mp3");
    audio_lib.audio_files.insert("clack".to_string(), music);

}
