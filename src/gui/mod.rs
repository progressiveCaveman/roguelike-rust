use rltk::{Rltk, Point, VirtualKeyCode, RGB, RGBA};
use hecs::*;
use resources::*;
use crate::player::get_player_map_knowledge;
use crate::{WINDOWWIDTH, GameMode, State};
use crate::components::{CombatStats, Name, Position, Viewshed, Fire};
use crate::gamelog::GameLog;
use crate::map::{OFFSET_Y};
use crate::map::Map;

pub mod camera;
pub use camera::*;

pub mod gui_menus;
pub use gui_menus::*;

/*
Render strategy:
Background color shows material
Glyph shows entity 
glyph color is set by entity in general
Background color is modified by tile status such as gas, light, or fire
Glyph color is modified by some statuses?
 */

// https://dwarffortresswiki.org/index.php/Character_table

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {Cancel, NoResponse, Selected}

pub struct Palette;
impl Palette {
    pub const MAIN_BG: rltk::RGBA = RGBA{r: 0., g: 0., b: 0., a: 0.};
    pub const MAIN_FG: rltk::RGBA = RGBA{r: 0.5, g: 0.5, b: 0.5, a: 1.};
    pub const COLOR_PURPLE: rltk::RGBA = RGBA{r: 1., g: 0., b: 1., a: 1.};
    pub const COLOR_RED: rltk::RGBA = RGBA{r: 1., g: 0., b: 0., a: 1.};
    pub const COLOR_GREEN: rltk::RGBA = RGBA{r: 0., g:0.7, b:0., a: 1.};
    pub const COLOR_GREEN_DARK: rltk::RGBA = RGBA{r: 0., g: 0.2, b: 0., a: 1.};
    pub const COLOR_3: rltk::RGBA = RGBA{r: 0.7, g: 0.2, b: 0.2, a: 1.};
    pub const COLOR_4: rltk::RGBA = RGBA{r: 0.7, g:0.7, b:0., a: 1.};
    pub const COLOR_AMBER: rltk::RGBA = RGBA{r: 1., g:0.74, b:0., a: 1.};
    pub const COLOR_WOOD: RGBA = RGBA{r: 0.45, g:0.38, b:0.26, a: 1.};
    pub const COLOR_DIRT: RGBA = RGBA{r: 0.6, g:0.46, b:0.32, a: 1.};
    pub const COLOR_WATER: RGBA = RGBA{r: 0.0, g:0.0, b:0.82, a: 1.};
    pub const COLOR_FIRE: RGBA = RGBA{r: 0.88, g:0.34, b:0.13, a: 1.};
    pub const COLOR_CEDAR: RGBA = RGBA{r: 0.39, g:0.22, b:0.17, a: 1.};
    pub const COLOR_CLEAR: RGBA = RGBA{r: 0., g:0., b:0., a: 0.};
    pub const FACTION_COLORS: [RGBA; 2] = [RGBA{r: 1.0, g:0., b:0., a: 1.}, RGBA{r: 0.0, g:0.0, b:1.0, a: 1.}];
}

pub fn draw_gui(gs: &State, ctx: &mut Rltk) {

    let world = &gs.world;
    let res = &gs.resources;

    let player_id: &Entity = &res.get::<Entity>().unwrap();
    let player_stats = world.get::<CombatStats>(*player_id).unwrap();
    let hp_gui = format!("{} / {} HP", player_stats.hp, player_stats.max_hp);
    let map = res.get::<Map>().unwrap();
    let turn = res.get::<i32>().unwrap();

    // horizontal line
    ctx.print_color(0, 10, Palette::MAIN_FG, Palette::MAIN_BG, "─".repeat(WINDOWWIDTH));

    // player stats
    ctx.print_color(0, 1, Palette::MAIN_FG, Palette::MAIN_BG, hp_gui);
    ctx.print_color(0, 2, Palette::MAIN_FG, Palette::MAIN_BG, &format!("Turn: {}", *turn));
    ctx.print_color(0, 9, Palette::MAIN_FG, Palette::MAIN_BG, format!("Depth: {}", map.depth));


    // On fire display
    let fire = world.get::<Fire>(*player_id);
    match fire {
        Ok(_) => {
            ctx.print_color(0, 2, Palette::MAIN_FG, Palette::COLOR_FIRE, "FIRE"); 
        },
        Err(_) => {},
    }

    for y in 0..10 {
        ctx.print_color(20, y, Palette::MAIN_FG, Palette::MAIN_BG, "│");
    }
    ctx.print_color(20, 10, Palette::MAIN_FG, Palette::MAIN_BG, "┴");

    // message log
    let log = res.get::<GameLog>().unwrap();
    let mut y = 1;
    for m in log.messages.iter().rev() {
        if y < 9 {
            ctx.print_color(21, y, Palette::MAIN_FG, Palette::MAIN_BG, m);
        }
        y += 1;
    }

    // draw mouse pos
    let mouse_pos = ctx.mouse_pos();
    if mouse_pos != (0, 0) {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_3);
    }
    draw_tooltips(gs, ctx);
}

pub fn draw_tooltips(gs: &State, ctx: &mut Rltk) {
    let world = &gs.world;
    let res = &gs.resources;

    let (min_x, _max_x, min_y, _max_y) = camera::get_map_coords_for_screen(world, res, ctx);
    let map = res.get::<Map>().unwrap();
    let gamemode = *res.get::<GameMode>().unwrap();

    let mouse_pos = ctx.mouse_pos();
    let mut map_mouse_pos = map.transform_mouse_pos(mouse_pos);
    map_mouse_pos.0 += min_x;
    map_mouse_pos.1 += min_y;
    if map_mouse_pos.0 >= map.width-1 || map_mouse_pos.1 >= map.height-1 || map_mouse_pos.0 < 1 || map_mouse_pos.1 < 1 { return; }
    
    let idx = map.xy_idx(map_mouse_pos.0, map_mouse_pos.1);
    if gamemode != GameMode::Sim && get_player_map_knowledge(gs).contains_key(&idx) { return; }

    let mut tooltip: Vec<String> = Vec::new();

    for (_id, (name, pos)) in world.query::<(&Name, &Position)>().iter() {
        for pos in pos.ps.iter() {
            if pos.x == map_mouse_pos.0 && pos.y == map_mouse_pos.1 {
                tooltip.push(name.name.to_string());
            }
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 { width = s.len() as i32; }
        }
        width += 3;

        let mut sign = 1;
        let mut arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
        let mut left_x = mouse_pos.0 + 4;
        let mut y = mouse_pos.1;
        if mouse_pos.0 > map.width / 2 {
            sign = -1;
            arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            left_x = mouse_pos.0 - width;
        }

        if sign == -1 {ctx.fill_region(rltk::Rect{x1: left_x, x2: left_x - 3 + width, y1: y, y2: y + tooltip.len() as i32 - 1}, rltk::to_cp437(' '), Palette::MAIN_FG, Palette::COLOR_3);}
        else {ctx.fill_region(rltk::Rect{x1: left_x - 1, x2: left_x + width - 4, y1: y, y2: y + tooltip.len() as i32 - 1}, rltk::to_cp437(' '), Palette::MAIN_FG, Palette::COLOR_3);}

        for s in tooltip.iter() {
            ctx.print_color(left_x, y, Palette::MAIN_FG, Palette::COLOR_3, s);
            y += 1;
        }
        ctx.print_color(arrow_pos.x, arrow_pos.y, Palette::MAIN_FG, Palette::COLOR_3, "->");
    }
}

pub fn ranged_target(world: &mut World, res: &mut Resources, ctx: &mut Rltk, range: i32) -> (ItemMenuResult, Option<Point>) {
    let map = res.get::<Map>().unwrap();
    let player_id = res.get::<Entity>().unwrap();
    let player_pos = res.get::<Point>().unwrap();
    ctx.print_color(5, 12, Palette::COLOR_PURPLE, Palette::MAIN_BG, "Select a target");

    let (min_x, max_x, min_y, max_y) = camera::get_map_coords_for_screen(world, res, ctx);

    let mut valid_cells: Vec<Point> = Vec::new();
    match world.get::<Viewshed>(*player_id) {
        Err(_e) => {return (ItemMenuResult::Cancel, None)},
        Ok(player_vs) => {
            for pt in player_vs.visible_tiles.iter() {
                let dist = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *pt);
                if dist as i32 <= range {
                    let screen_x = pt.x - min_x;
                    let screen_y = pt.y - min_y + OFFSET_Y as i32; // TODO why is offset needed here??
                    if screen_x > 1 && screen_x < (max_x - min_x)-1 && screen_y > 1 && screen_y < (max_y - min_y)-1 {
                        ctx.set_bg(screen_x, screen_y, RGB::named(rltk::BLUE));
                        valid_cells.push(*pt);
                    }
                    ctx.set_bg(screen_x, screen_y, Palette::COLOR_4);
                    valid_cells.push(*pt);
                }
            }
        }
    }

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;

    let mouse_pos = ctx.mouse_pos();
    let mut map_mouse_pos = map.transform_mouse_pos(mouse_pos);
    map_mouse_pos.0 += min_x;
    map_mouse_pos.1 += min_y;
    // let map_mouse_pos = (mouse_pos.0 - map::OFFSET_X as i32, mouse_pos.1 - map::OFFSET_Y as i32);
    let mut valid_target = false;
    for pt in valid_cells.iter() {
        if pt.x == map_mouse_pos.0 && pt.y == map_mouse_pos.1 { valid_target = true }
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_GREEN_DARK);
        if ctx.left_click { return (ItemMenuResult::Selected, Some(Point::new(map_mouse_pos.0, map_mouse_pos.1))) }
    }
    else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_RED);
        if ctx.left_click { return (ItemMenuResult::Cancel, None) }
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { return (ItemMenuResult::Cancel, None) },
                _ => (ItemMenuResult::NoResponse, None)
            }
        }
    }
}