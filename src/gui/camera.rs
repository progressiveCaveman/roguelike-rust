use hecs::World;
use resources::Resources;
use crate::{gui::Palette, Scale, map::{OFFSET_X, OFFSET_Y, TileType}, GameMode, SCALE, components::{Player, Renderable}, MAPHEIGHT, MAPWIDTH, State, player::{get_player_map_knowledge, get_player_viewshed}};

use super::{Map,Position};
use rltk::{Point, Rltk, RGB, RGBA};

const SHOW_BOUNDARIES : bool = true;
const RENDER_DJIKSTRA: bool = false;

pub fn get_map_coords_for_screen(world: &World, res: &Resources, ctx: &mut Rltk) -> (i32, i32, i32, i32) {
    let player_pos = &res.get::<Point>().unwrap();
    let (mut x_chars, mut y_chars) = ctx.get_char_size();
    x_chars -= (OFFSET_X as f32 / SCALE).ceil() as u32;
    y_chars -= (OFFSET_Y as f32 / SCALE).ceil() as u32;

    let center_x = (x_chars as f32 / 2.0) as i32;
    let center_y = (y_chars as f32 / 2.0) as i32;

    let mut min_x = player_pos.x - center_x;
    let mut max_x = player_pos.x + center_x;
    let mut min_y = player_pos.y - center_y;
    let mut max_y = player_pos.y + center_y;

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

pub fn render_camera(gs: &State, ctx : &mut Rltk) {
    let world = &gs.world;
    let res = &gs.resources;

    ctx.set_active_console(1);

    // make console clear so it doesn't block
    // let (xsize, ysize) = ctx.get_char_size();
    // for x in 0..xsize {
    //     for y in 0..ysize {
    //         ctx.set(x, y, RGBA::from_u8(0, 0, 0, 255), RGBA::from_u8(0, 0, 0, 255), rltk::to_cp437(' '));                
    //     }
    // }

    let scale: f32 = SCALE;
    let xoff:f32 = (OFFSET_X as f32 / scale).ceil();
    let yoff:f32 = (OFFSET_Y as f32 / scale).ceil();
    let size = ctx.get_char_size();

    let map = res.get::<Map>().unwrap();
    let gamemode = *res.get::<GameMode>().unwrap();
    let player_knowledge = get_player_map_knowledge(gs);

    let (min_x, max_x, min_y, max_y) = get_map_coords_for_screen(world, res, ctx);

    let map_width = map.width;
    let map_height = map.height;

    let mut y = yoff as usize;
    for ty in min_y ..= max_y {
        let mut x = xoff as usize;
        for tx in min_x ..= max_x {
            if tx >= 0 && tx < map_width && ty >= 0 && ty < map_height {
                let idx = map.xy_idx(tx, ty);
                let p = Point { x: tx, y: ty };
                if gamemode == GameMode::Sim || player_knowledge.contains_key(&idx) {
                    let (glyph, mut fg, mut bg) = get_tile_glyph(idx, &*map);

                    if gamemode != GameMode::Sim && !get_player_viewshed(gs).is_visible(p) {
                        fg.scale(0.5);
                        bg.scale(0.5);
                    }

                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(x, y, RGB::named(rltk::GRAY), RGB::named(rltk::BLACK), rltk::to_cp437('·'));                
            }
            x += 1;
        }
        y += 1;
    }

    // ctx.set_active_console(1);

    // draw entities
    for (_, (pos, render, player)) in world.query::<(&Position, &Renderable, Option<&Player>)>().iter() {
        if let Some(_) = player {
            if gamemode == GameMode::Sim { continue; }
        }

        for pos in pos.ps.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            if pos.y > min_y - 1 && pos.x > min_x - 1 && (gamemode == GameMode::Sim || get_player_viewshed(gs).is_visible(*pos) ) { 
                let (_, _, bgcolor) = get_tile_glyph(idx, &*map);

                let entity_screen_x = xoff as i32 + pos.x - min_x;
                let entity_screen_y = yoff as i32 + pos.y - min_y;
                if entity_screen_x > -1 && entity_screen_x < size.0 as i32 && entity_screen_y > 0 && entity_screen_y < size.1 as i32 {
                    ctx.set(entity_screen_x, entity_screen_y, render.fg, bgcolor, render.glyph);
                }
            }
        }
    }

    ctx.set_active_console(0);
}

fn get_tile_glyph(idx: usize, map : &Map) -> (rltk::FontCharType, RGBA, RGBA) {
    let mut glyph = rltk::to_cp437(' ');
    let mut fg = Palette::MAIN_FG;
    let mut bg = Palette::MAIN_BG;

    match map.tiles[idx] {
        TileType::Floor => {
            fg = Palette::COLOR_GREEN_DARK;
            if RENDER_DJIKSTRA && map.dijkstra_map[idx] >= 0.0 {
                let val = (map.dijkstra_map[idx] % 10.0) as u8;
                let cha = (val + b'0') as char;
                glyph = rltk::to_cp437(cha);
            }else{
                glyph = rltk::to_cp437('·');
            }
        }
        TileType::Wall => {
            fg = Palette::MAIN_FG;
            glyph = rltk::to_cp437('#');
        }
        TileType::StairsDown => {
            fg = Palette::MAIN_FG;
            glyph = rltk::to_cp437('>');
        }
        TileType::StairsUp => {
            fg = Palette::MAIN_FG;
            glyph = rltk::to_cp437('<');
        }
        TileType::Grass => {
            // fg = Palette::COLOR_GREEN;
            bg = Palette::COLOR_GREEN_DARK;
            // glyph = rltk::to_cp437('"');
        }
        TileType::Wheat => {
            fg = Palette::COLOR_AMBER;
            // let gs = vec!['|', '{', '}'];
            // let c = gs.choose(&mut rand::thread_rng()).unwrap();
            let c = '{';
            glyph = rltk::to_cp437(c);
        }
        TileType::Dirt => {
            fg = Palette::COLOR_DIRT;
            glyph = rltk::to_cp437('.');
        }
        TileType::Water => {
            fg = Palette::COLOR_WATER;
            glyph = rltk::to_cp437('~');
        }
        TileType::WoodWall => {
            fg = Palette::COLOR_WOOD;
            glyph = rltk::to_cp437('#');
        },
        TileType::WoodDoor => {
            fg = Palette::COLOR_WOOD;
            glyph = rltk::to_cp437('+');
        },
        TileType::WoodFloor => {
            fg = Palette::COLOR_WOOD;
            glyph = rltk::to_cp437('.');
        },
    }

    if map.fire_turns[idx] > 0 { // TODO check if player knows about fire
        bg = Palette::COLOR_FIRE;
        glyph = rltk::to_cp437('^');
    }

    // let f1val = map.influence_maps[0][idx];
    // fg.scale(f1val);

    (glyph, fg, bg)
}

// fn wall_glyph(map : &Map, x: i32, y:i32) -> rltk::FontCharType {
//     if x < 1 || x > map.width-2 || y < 1 || y > map.height-2 as i32 { return 35; }
//     let mut mask : u8 = 0;

//     if is_revealed_and_wall(map, x, y - 1) { mask +=1; }
//     if is_revealed_and_wall(map, x, y + 1) { mask +=2; }
//     if is_revealed_and_wall(map, x - 1, y) { mask +=4; }
//     if is_revealed_and_wall(map, x + 1, y) { mask +=8; }

//     match mask {
//         0 => { 9 } // Pillar because we can't see neighbors
//         1 => { 186 } // Wall only to the north
//         2 => { 186 } // Wall only to the south
//         3 => { 186 } // Wall to the north and south
//         4 => { 205 } // Wall only to the west
//         5 => { 188 } // Wall to the north and west
//         6 => { 187 } // Wall to the south and west
//         7 => { 185 } // Wall to the north, south and west
//         8 => { 205 } // Wall only to the east
//         9 => { 200 } // Wall to the north and east
//         10 => { 201 } // Wall to the south and east
//         11 => { 204 } // Wall to the north, south and east
//         12 => { 205 } // Wall to the east and west
//         13 => { 202 } // Wall to the east, west, and south
//         14 => { 203 } // Wall to the east, west, and north
//         15 => { 206 }  // ╬ Wall on all sides
//         _ => { 35 } // We missed one?
//     }
// }

// fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
//     let idx = map.xy_idx(x, y);
//     map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
// }