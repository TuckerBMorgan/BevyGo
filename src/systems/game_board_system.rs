use std::hash::Hash;
use bevy::prelude::*;

#[derive(Copy, Clone, PartialEq,Hash, Reflect)]
pub enum Tile {
    Empty,
    Black,
    White
}

#[derive(Copy, Clone, PartialEq,Hash, Reflect)]
pub struct TileData {
    pub tile_state: Tile,
    pub is_alive: bool
}

impl Default for TileData {
    fn default() -> TileData {
        TileData {
            tile_state: Tile::Empty,
            is_alive: false
        }
    }
}

pub struct TileSpriteData {
    pub index: usize
}

impl TileSpriteData {
    pub fn new(index: usize) -> TileSpriteData {
        TileSpriteData {
            index
        }
    }
}

#[derive(Hash, Reflect)]
#[reflect(Hash)]
pub struct GameState {
    pub game_board: Vec<TileData>,
    pub current_player: Tile,
    pub current_player_id: usize
}

impl Default for GameState {
    fn default() -> GameState {
        GameState {
            game_board:  [TileData::default();81].to_vec(),
            current_player: Tile::White,
            current_player_id: 0
        }
    }
}

impl GameState {
    pub fn make_move(&mut self, index: usize) {
        self.game_board[index].tile_state = self.current_player;
        match self.current_player {
            Tile::White => {
                self.current_player = Tile::Black;
            }
            _ => {
                self.current_player = Tile::White;
            }
        }

        let dead_indicies = apply_life_and_death_rules_to_board(&mut self.game_board);
        for i in dead_indicies {
            self.game_board[i].tile_state = Tile::Empty;
        }
    }
}

fn grant_life_to_stone(index: usize,  board: &mut Vec<TileData>) {
    if board[index].is_alive || board[index].tile_state == Tile::Empty {
        return;
    }
    board[index].is_alive = true;

    let index_state = board[index].tile_state;
    let neighbor_indices = neighbor_indices(index);
    for ii in neighbor_indices {
        if board[ii].tile_state == index_state {
            grant_life_to_stone(ii, board);
        }
    }
}

pub fn apply_life_and_death_rules_to_board(board: &mut Vec<TileData>) -> Vec<usize> {
    for i in 0..board.len() {
        board[i].is_alive = false;
    }

    for i in 0..board.len() {
        if board[i].tile_state == Tile::Empty {
            let neighbor_indices = neighbor_indices(i);
            for index in neighbor_indices {
                grant_life_to_stone(index, board);
            }
        }
    }

    let mut return_indices = vec![];
    for i in 0..board.len() {
        if board[i].is_alive == false {
            return_indices.push(i);
        }
    }
    
    return_indices
}

pub fn neighbor_indices(index: usize) -> Vec<usize> {
    let index = index as i32;
    let mut up_down : Vec<usize> = vec![index - 9, index + 9].iter().filter(|x| **x >= 0 && **x < 81).filter(|x| ((**x) % 9) == (index%9)).map(|x|(*x) as usize).collect();
    let left_right : Vec<usize> = vec![index - 1, index + 1, ].iter().filter(|x| **x >= 0 && **x < 81).filter(|x| ((**x) / 9) == (index/9)).map(|x|(*x) as usize).collect();
    up_down.extend(left_right);
    return up_down;
}
