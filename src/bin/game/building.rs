use tcod::console::*;
use tcod::colors::Color;

use crate::game::Coord;
use crate::game::Resources;

#[derive(Clone)]
pub struct Building {
    pub btype:   usize,
    pub topleft: Coord,
    pub dimxy:   Coord,
    pub cost:    Resources,
    color:   Color,
    pub update_fn: fn(&mut Resources, i32),
}

impl Building {
    pub fn new_type<T>(btype: T, dimxy: Coord, cost: Resources, color: Color, f: fn(&mut Resources, i32)) -> Self 
    where T: std::convert::Into<usize> 
    {
        Building{ 
            btype: btype.into(), 
            topleft: Coord::new( 0, 0 ), 
            dimxy: dimxy, 
            cost: cost, 
            color: color, 
            update_fn:f, 
        }    
    }

    pub fn new_token(self, topleft: Coord) -> Self {
        let mut t = self.clone(); 
        t.topleft = topleft;
        t
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        if self.dimxy.x == 1 {
            // church
            con.put_char(self.topleft.x, self.topleft.y, 197 as char, BackgroundFlag::None);
            con.put_char(self.topleft.x, self.topleft.y+1, 179 as char, BackgroundFlag::None);
            return
        }
        // others
        for y in self.topleft.y..self.topleft.y + self.dimxy.y {
            for x in self.topleft.x..self.topleft.x + self.dimxy.x {
                if x == self.topleft.x {
                    if y == self.topleft.y {
                        con.put_char(x, y, 201 as char, BackgroundFlag::None);
                        continue
                    }
                    if y == self.topleft.y + self.dimxy.y - 1{
                        con.put_char(x, y, 200 as char, BackgroundFlag::None);
                        continue
                    }
                    con.put_char(x, y, 186 as char, BackgroundFlag::None);
                    continue
                }
                if x == self.topleft.x + self.dimxy.x - 1{
                    if y == self.topleft.y {
                        con.put_char(x, y, 187 as char, BackgroundFlag::None);
                        continue
                    }
                    if y == self.topleft.y + self.dimxy.y - 1{
                        con.put_char(x, y, 188 as char, BackgroundFlag::None);
                        continue
                    }
                    con.put_char(x, y, 186 as char, BackgroundFlag::None);
                    continue
                }
                if y == self.topleft.y || y == self.topleft.y + self.dimxy.y - 1 {
                    con.put_char(x, y, 205 as char, BackgroundFlag::None);
                    continue
                }
                con.put_char(x, y, 178 as char, BackgroundFlag::None);
            }
        }
    }
}
