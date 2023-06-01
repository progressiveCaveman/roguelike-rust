use std::collections::HashMap;

use engine::{
    components::Item,
    effects::{add_effect, EffectType},
    map::Map,
    player,
    utils::{dir_to_point, PPoint, PlayerID},
    GameMode, GameSettings,
};
use rltk::{Rltk, VirtualKeyCode};
use shipyard::{Get, UniqueView, UniqueViewMut, View, World, EntityId};

use crate::RunState;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Command {
    None,
    Move {
        dir: i32,
    },
    ShowInventory,
    Wait,
    Escape,
    Get,
    Explore,
    RevealMap,
    Fireball,
    UseStairs
}

impl Command {
    fn execute(&self, world: &World, creator: Option<EntityId>) -> RunState {
        let map = world.borrow::<UniqueView<Map>>().unwrap();

        let player_pos = world.borrow::<UniqueView<PPoint>>().unwrap().0;
        let player_pos_idx = map.point_idx(player_pos);

        // return RunState::AwaitingInput to ignore input, RunState::PlayerTurn to advance engine
        return match self {
            Command::None => {
                RunState::AwaitingInput
            },
            Command::Move { dir } => {
                // hold shift to move by 10 squares at a time
                let movemod = 1;

                let mut dir_targets: HashMap<i32, usize> = HashMap::new();
                dir_targets.insert(1, map.point_idx(dir_to_point(player_pos, 1, movemod)));
                dir_targets.insert(2, map.point_idx(dir_to_point(player_pos, 2, movemod)));
                dir_targets.insert(3, map.point_idx(dir_to_point(player_pos, 3, movemod)));
                dir_targets.insert(4, map.point_idx(dir_to_point(player_pos, 4, movemod)));
                dir_targets.insert(6, map.point_idx(dir_to_point(player_pos, 6, movemod)));
                dir_targets.insert(7, map.point_idx(dir_to_point(player_pos, 7, movemod)));
                dir_targets.insert(8, map.point_idx(dir_to_point(player_pos, 8, movemod)));
                dir_targets.insert(9, map.point_idx(dir_to_point(player_pos, 9, movemod)));

                add_effect(
                    creator,
                    EffectType::MoveOrAttack {
                        tile_idx: dir_targets[dir],
                    },
                );

                RunState::PlayerTurn
            },
            Command::ShowInventory => {
                RunState::ShowInventory
            },
            Command::Wait => {
                add_effect(creator, EffectType::Wait {}); //todo is this weird on sim mode? 
                RunState::PlayerTurn
            },
            Command::Escape => {
                RunState::EscPressed
            },
            Command::Get => {
                world.run(|vitem: View<Item>| {
                    for e in map.tile_content[player_pos_idx].iter() {
                        if let Ok(_) = vitem.get(*e) {
                            add_effect(creator, EffectType::PickUp { entity: *e });
                        }
                    }
                });

                RunState::PlayerTurn
            },
            Command::Explore => {
                add_effect(creator, EffectType::Explore {});

                RunState::PlayerTurn
            },
            Command::RevealMap => {
                player::reveal_map(&world);

                RunState::PlayerTurn
            },
            Command::Fireball => {
                dbg!("fireball is broken");
                RunState::AwaitingInput
                // RunState::ShowTargeting {
                //     range: 6,
                //     item: world.run(|mut store: AllStoragesViewMut| {
                //         entity_factory::tmp_fireball(&mut store)
                //     }),
                // }
            },
            Command::UseStairs => {
                if player::try_next_level(&world) {
                    RunState::NextLevel
                }else {
                    RunState::AwaitingInput
                }
            },
        };
    }
}

pub fn can_use_command(gamemode: GameMode, command: Command) -> bool {
    match gamemode {
        GameMode::NotSelected => unreachable!(),
        GameMode::Sim => {
            match command {
                Command::None => true,
                Command::Move { .. } => true,
                Command::ShowInventory => false,
                Command::Wait => true,
                Command::Escape => true,
                Command::Get { .. } => false,
                Command::Explore => false,
                Command::RevealMap => false,
                Command::Fireball => false,
                Command::UseStairs => false,   
            }
        },
        GameMode::RL => {
            match command {
                Command::None => true,
                Command::Move { .. } => true,
                Command::ShowInventory => true,
                Command::Wait => true,
                Command::Escape => true,
                Command::Get { .. } => true,
                Command::Explore => true,
                Command::RevealMap => true,
                Command::Fireball => true,
                Command::UseStairs => true,
            }
        },
    }
}

pub fn map_keys(ctx: &Rltk) -> Command {
    return match ctx.key {
        None => Command::None,
        Some(key) => match key {
            VirtualKeyCode::Left => Command::Move { dir: 4 },
            VirtualKeyCode::Right => Command::Move { dir: 6 },
            VirtualKeyCode::Up => Command::Move { dir: 8 },
            VirtualKeyCode::Down => Command::Move { dir: 2 },
            VirtualKeyCode::Y => Command::Move { dir: 7 },
            VirtualKeyCode::U => Command::Move { dir: 9 },
            VirtualKeyCode::N => Command::Move { dir: 3 },
            VirtualKeyCode::B => Command::Move { dir: 1 },
            VirtualKeyCode::G => Command::Get,
            VirtualKeyCode::X => Command::Explore,
            VirtualKeyCode::R => Command::RevealMap,
            VirtualKeyCode::F => Command::Fireball,
            VirtualKeyCode::I => Command::ShowInventory,
            VirtualKeyCode::W => Command::Wait,
            VirtualKeyCode::Escape => Command::Escape,
            VirtualKeyCode::Period => Command::UseStairs,
            _ => Command::None,
        },
    }
}

pub fn handle_input(world: &World, ctx: &Rltk) -> RunState {
    let command = map_keys(ctx);

    let settings = world.borrow::<UniqueView<GameSettings>>().unwrap();
    let player_id = world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;

    let can_use = can_use_command(settings.mode, command);

    if can_use {
        return command.execute(world, Some(player_id));
    }

    return RunState::AwaitingInput;
}