use bevy::{
    prelude::*,
};

pub struct Scoreboard {
    pub score: usize,
}

pub fn scoreboard_system(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut().unwrap();
    text.sections[0].value = format!("Score: {}", scoreboard.score);
}