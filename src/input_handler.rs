

pub fn handle_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    let game_mode: GameMode = *gs.resources.get::<GameMode>().unwrap();

    if game_mode == GameMode::RL {
        
    }
}





pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    let player_id: Entity = *gs.resources.get::<Entity>().unwrap();

    match ctx.key {
        None => { return RunState::AwaitingInput }
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_entity(player_id, -1, 0, gs),
            VirtualKeyCode::Right => try_move_entity(player_id, 1, 0, gs),
            VirtualKeyCode::Up => try_move_entity(player_id, 0, -1, gs),
            VirtualKeyCode::Down => try_move_entity(player_id, 0, 1, gs),
            VirtualKeyCode::Y => try_move_entity(player_id, -1, -1, gs),
            VirtualKeyCode::U => try_move_entity(player_id, 1, -1, gs),
            VirtualKeyCode::N => try_move_entity(player_id, 1, 1, gs),
            VirtualKeyCode::B => try_move_entity(player_id, -1, 1, gs),
            VirtualKeyCode::G => get_item(&mut gs.world, &mut gs.resources),
            VirtualKeyCode::X => autoexplore(gs),
            VirtualKeyCode::R => reveal_map(gs),
            VirtualKeyCode::F => return RunState::ShowTargeting { range: 6, item: entity_factory::tmp_fireball(&mut gs.world) },
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::W => return skip_turn(&mut gs.world, &mut gs.resources),
            VirtualKeyCode::Escape => return RunState::SaveGame,
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.world, &mut gs.resources) { return RunState::NextLevel; }
            }
            _ => { return RunState::AwaitingInput }
        }
    }
    RunState::PlayerTurn
}
