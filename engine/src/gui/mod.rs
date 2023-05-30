use crate::components::Viewshed;
use crate::map::Map;
use crate::utils::{PPoint, PlayerID};
use crate::{State, MAPHEIGHT, MAPWIDTH, SCALE};
use rltk::{Point, Rltk, VirtualKeyCode, RGB, RGBA};
use shipyard::{Get, UniqueView, View};

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

pub const OFFSET_X: usize = 31;
pub const OFFSET_Y: usize = 11;

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub struct Palette;
impl Palette {
    pub const MAIN_BG: rltk::RGBA = RGBA {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 0.,
    };
    pub const MAIN_FG: rltk::RGBA = RGBA {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.,
    };
    pub const COLOR_PURPLE: rltk::RGBA = RGBA {
        r: 1.,
        g: 0.,
        b: 1.,
        a: 1.,
    };
    pub const COLOR_RED: rltk::RGBA = RGBA {
        r: 1.,
        g: 0.,
        b: 0.,
        a: 1.,
    };
    pub const COLOR_GREEN: rltk::RGBA = RGBA {
        r: 0.,
        g: 0.7,
        b: 0.,
        a: 1.,
    };
    pub const COLOR_GREEN_DARK: rltk::RGBA = RGBA {
        r: 0.,
        g: 0.2,
        b: 0.,
        a: 1.,
    };
    pub const COLOR_3: rltk::RGBA = RGBA {
        r: 0.7,
        g: 0.2,
        b: 0.2,
        a: 1.,
    };
    pub const COLOR_4: rltk::RGBA = RGBA {
        r: 0.7,
        g: 0.7,
        b: 0.,
        a: 1.,
    };
    pub const COLOR_AMBER: rltk::RGBA = RGBA {
        r: 1.,
        g: 0.74,
        b: 0.,
        a: 1.,
    };
    pub const COLOR_WOOD: RGBA = RGBA {
        r: 0.45,
        g: 0.38,
        b: 0.26,
        a: 1.,
    };
    pub const COLOR_DIRT: RGBA = RGBA {
        r: 0.6,
        g: 0.46,
        b: 0.32,
        a: 1.,
    };
    pub const COLOR_WATER: RGBA = RGBA {
        r: 0.0,
        g: 0.0,
        b: 0.82,
        a: 1.,
    };
    pub const COLOR_FIRE: RGBA = RGBA {
        r: 0.88,
        g: 0.34,
        b: 0.13,
        a: 1.,
    };
    pub const COLOR_CEDAR: RGBA = RGBA {
        r: 0.39,
        g: 0.22,
        b: 0.17,
        a: 1.,
    };
    pub const COLOR_CLEAR: RGBA = RGBA {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 0.,
    };
    pub const FACTION_COLORS: [RGBA; 2] = [
        RGBA {
            r: 1.0,
            g: 0.,
            b: 0.,
            a: 1.,
        },
        RGBA {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.,
        },
    ];
}

pub fn ranged_target(gs: &State, ctx: &mut Rltk, range: i32) -> (ItemMenuResult, Option<Point>) {
    let map = gs.world.borrow::<UniqueView<Map>>().unwrap();
    let player_id = gs.world.borrow::<UniqueView<PlayerID>>().unwrap().0;
    let player_pos = gs.world.borrow::<UniqueView<PPoint>>().unwrap().0;
    ctx.print_color(
        5,
        12,
        Palette::COLOR_PURPLE,
        Palette::MAIN_BG,
        "Select a target",
    );

    let (min_x, max_x, min_y, max_y) = get_map_coords_for_screen(player_pos, ctx);

    let mut valid_cells: Vec<Point> = Vec::new();
    let vvs = gs.world.borrow::<View<Viewshed>>().unwrap();
    match vvs.get(player_id) {
        Err(_e) => return (ItemMenuResult::Cancel, None),
        Ok(player_vs) => {
            for pt in player_vs.visible_tiles.iter() {
                let dist = rltk::DistanceAlg::Pythagoras.distance2d(player_pos, *pt);
                if dist as i32 <= range {
                    let screen_x = pt.x - min_x + OFFSET_X as i32;
                    let screen_y = pt.y - min_y + OFFSET_Y as i32; // TODO why is offset needed here??
                    if screen_x > 1
                        && screen_x < (max_x - min_x) - 1
                        && screen_y > 1
                        && screen_y < (max_y - min_y) - 1
                    {
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
        if pt.x == map_mouse_pos.0 && pt.y == map_mouse_pos.1 {
            valid_target = true
        }
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_GREEN_DARK);
        if ctx.left_click {
            return (
                ItemMenuResult::Selected,
                Some(Point::new(map_mouse_pos.0, map_mouse_pos.1)),
            );
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, Palette::COLOR_RED);
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => return (ItemMenuResult::Cancel, None),
            _ => (ItemMenuResult::NoResponse, None),
        },
    }
}

pub fn get_map_coords_for_screen(focus: Point, ctx: &mut Rltk) -> (i32, i32, i32, i32) {
    let (mut x_chars, mut y_chars) = ctx.get_char_size();
    x_chars -= (OFFSET_X as f32 / SCALE).ceil() as u32;
    y_chars -= (OFFSET_Y as f32 / SCALE).ceil() as u32;

    let center_x = (x_chars as f32 / 2.0) as i32;
    let center_y = (y_chars as f32 / 2.0) as i32;

    let mut min_x = focus.x - center_x;
    let mut max_x = focus.x + center_x;
    let mut min_y = focus.y - center_y;
    let mut max_y = focus.y + center_y;

    let w = MAPWIDTH as i32;
    let h = MAPHEIGHT as i32;

    // Now check for borders, don't scroll past map edge
    if min_x < 0 {
        max_x -= min_x;
        min_x = 0;
    } else if max_x > w {
        min_x -= max_x - w;
        max_x = w - 1;
    }

    if min_y < 0 {
        max_y += 0 - min_y;
        min_y = 0;
    } else if max_y > h {
        min_y -= max_y - h;
        max_y = h - 1;
    }

    (min_x, max_x, min_y, max_y)
}
