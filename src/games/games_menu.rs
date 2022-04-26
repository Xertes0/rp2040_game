use cortex_m::delay::Delay;

use crate::inputs::Inputs;
use crate::menu::{Menu, MenuOption};
use crate::pcd8544::PCD8544;

use super::snake::SnakeGame;

#[derive(Clone, Copy)]
enum GameSelected {
    Snake, PingPong, Quit
}

pub struct GamesMenu;

impl GamesMenu {
    pub fn run(pcd: &mut PCD8544, inputs: &mut Inputs, delay: &mut Delay) {
        let mut game_menu = Menu::new([
            MenuOption::new(GameSelected::Snake, "Wensz"),
            MenuOption::new(GameSelected::PingPong, "PingPong"),
            MenuOption::new(GameSelected::Quit, "Wyjdz"),
        ]);

        match game_menu.run(pcd, inputs, delay) {
            GameSelected::Snake => {
                let mut snake_game = SnakeGame::new(pcd, inputs, delay);
                snake_game.run();
            },
            GameSelected::PingPong => {

            },
            GameSelected::Quit => {

            }
        }
    }
}
