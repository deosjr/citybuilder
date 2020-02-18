use tcod::console::*;
use tcod::colors::*;
use std::collections::HashMap;

use building::Building;
pub mod building;
pub mod path;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum

pub struct Tcod {
    root: Root,
    con: Offscreen,
}

pub fn get_tcod(game: &Game) -> Tcod {
    tcod::system::set_fps(LIMIT_FPS);

    let root = Root::initializer()
        //.font("arial10x10.png", FontLayout::Tcod)
        .font("terminal12x12_gs_ro.png", FontLayout::AsciiInRow)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("City builder")
        .init();
    
    let con = Offscreen::new(game.mapxy.x, game.mapxy.y);

    Tcod { root, con }
}

pub fn do_loop(mut tcod: Tcod, game: &mut Game) {
    let mut i = 0;
    while !tcod.root.window_closed() {
        tcod.con.clear();
        game.render_all(&mut tcod);
        tcod.root.flush();

        game.update_buildings(i);

        // handle keys and exit game if needed
        let exit = handle_keys(&mut tcod, game);
        if exit {
            break;
        }
        i += 1;
    }
}

fn handle_keys(tcod: &mut Tcod, game: &mut Game) -> bool { //, player: &mut Object) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    use tcod::input::Mouse;
    use tcod::input::Event;

    let mut mouse: Mouse = Default::default();
    let mut key: Key = Default::default();
    match tcod::input::check_for_event(tcod::input::MOUSE | tcod::input::KEY_PRESS) {
        Some((_, Event::Mouse(m))) => mouse = m,
        Some((_, Event::Key(k))) => key = k,
        _ => {},
    }

    if mouse.cx != 0 && mouse.cy != 0 {
        game.mousexy.x = mouse.cx as i32;
        game.mousexy.y = mouse.cy as i32;
    }

    if mouse.lbutton {
        if game.mousexy.y > game.mapxy.y {
            // selecting a building in the bottom of the screen
            let x = (game.mousexy.x as usize - 1) / 4;
            if x < game.buildingtypes.len() {
                game.draw = Draw::Building(game.buildingtypes[x].btype);
            }
        } else {
            // drawing a building on the map
            match game.draw {
                Draw::Building(u) => {
                    let newhouse = game.get_building_to_build(u);
                    if !game.is_blocked(&newhouse) && game.can_pay_for(&newhouse) {
                        game.add(newhouse);
                    }
                },
                Draw::Road(r) => {
                    match r {
                        Some(road_start) => {
                            match path::find_route(&game.map, road_start, game.mousexy) {
                                Some(path) => {
                                    if game.resources.covers(&game.road_cost) {
                                        // TODO check whole road cost
                                    }
                                    for c in path.iter() {
                                        if game.map.get(c).unwrap().road {
                                            continue
                                        }
                                        game.add_road(*c);
                                    }
                                    game.draw = Draw::Road(None);
                                },
                                None => {},
                            }
                        },
                        None => {
                            game.draw = Draw::Road(Some(game.mousexy));   
                        },
                    }
                },
                Draw::None => {},
            }
        }
    }
    if mouse.rbutton {
        game.draw = Draw::None;
    }

    match key {
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        },
        Key { code: Escape, .. } => return true, // exit game
        Key { code: Text, .. } => if key.text() == "r" { game.draw = Draw::Road(None) },

        // movement keys
        /*
        Key { code: Up, .. } => player.move_by(0, -1, game),
        Key { code: Down, .. } => player.move_by(0, 1, game),
        Key { code: Left, .. } => player.move_by(-1, 0, game),
        Key { code: Right, .. } => player.move_by(1, 0, game),
        */

        _ => {}
    };

    false
}

#[derive(PartialOrd,Ord,Clone,Copy,PartialEq,Eq,Hash)]
pub struct Coord {
    x: i32,
    y: i32,
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Self {
        Coord{ x, y }
    }
}

// creates a Resources struct which is a list of numbers
// equal in length to the number of resources in the game
// and sets the named resources to their values
macro_rules! resources {
    ( $( ($r:expr, $x:expr) ),* ) => {
        {
            #[allow(unused)] // in case of resources![]
            let mut temp_vec = vec![0; Resource::NumResources as usize];
            $(
                temp_vec[$r as usize] = $x;
            )*
            crate::game::Resources(temp_vec)
        }
    };
}

// declares the types of resources in the game
macro_rules! resource_types {
    ($($body:tt)*) => {
        as_item! {
            #[derive(Copy,Clone)]
            enum Resource { $($body)* , NumResources }
        }

        impl Into<usize> for Resource {
            fn into(self) -> usize {
                self as usize
            }
        }
    };
}

// declares the types of buildings in the game
macro_rules! building_types {
    ($($body:tt)*) => {
        as_item! {
            #[derive(Copy,Clone,PartialEq,Eq)]
            enum BuildingType { $($body)* , NumBuildingTypes }
        }

        impl Into<usize> for BuildingType {
            fn into(self) -> usize {
                self as usize
            }
        }
    };
}

macro_rules! as_item {
    ($i:item) => { $i };
}

#[derive(Clone)]
pub struct Resources(pub Vec<(i32)>);

impl Resources {
    pub fn get<T>(&self, r: T) -> i32 
    where T: std::convert::Into<usize> 
    {
        self.0[r.into()]
    }

    pub fn set<T>(&mut self, r: T, v: i32)
    where T: std::convert::Into<usize> 
    {
        self.0[r.into()] = v;
    }

    pub fn update<T>(&mut self, r: T, v: i32)
    where T: std::convert::Into<usize> 
    {
        let rusize = r.into();
        self.0[rusize] = self.0[rusize] + v;
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn covers(&self, other: &Resources) -> bool {
        self.0.iter().zip(other.0.iter()).filter(|(&x, &y)| x < y).collect::<Vec<(&i32, &i32)>>().len() == 0
    }

    pub fn substract(&mut self, cost: &Resources) -> Resources {
        // TODO inplace
        Resources(self.0.iter().zip(cost.0.iter()).map(|(x, y)| x-y).collect::<Vec<i32>>())
    }
}

#[derive(PartialEq, Eq)]
pub enum Draw {
    Building(usize),
    Road(Option<Coord>),
    None,
}

pub struct Game<'a> {
    map: Map<'a>, //HashMap<Coord, Tile>,
    buildings: Vec<Building>,
    buildingtypes: &'a [Building],
    draw: Draw, //Option<usize>
    pub resources: Resources,
    road_cost: Resources,
    mapxy: Coord,
    mousexy: Coord,
}

impl<'a> Game<'a> {
    pub fn new<T>(buildings: &'a [Building], mapx: i32, mapy: i32, num_resources: T, road_cost: Resources, coords: &'a mut HashMap<Coord,Tile>) -> Self 
    where T: std::convert::Into<usize> 
    {
        let game = Game{ 
            map: Map::new(coords), 
            buildings: vec![], 
            buildingtypes: buildings, 
            resources: Resources(vec![0; num_resources.into()]), 
            road_cost: road_cost,
            draw: Draw::None,
            mapxy: Coord::new(mapx, mapy),
            mousexy: Coord::new(0, 0),
        };
        game
    }

    fn is_blocked(&self, newobj: &Building) -> bool {
        let Coord{x, y} = newobj.topleft;
        let Coord{x: dx, y: dy} = newobj.dimxy;
        for x in x..(x+dx) {
            for y in y..(y+dy) {
                if x > self.mapxy.x || y > self.mapxy.y {
                    return true
                }
                if self.map.get(&Coord::new(x,y)).unwrap().blocked {
                    return true
                }
            }
        }
        return false
    }

    fn can_pay_for(&self, newobj: &Building) -> bool {
        self.resources.covers(&newobj.cost)
    }

    fn add(&mut self, newobj: Building) {
        self.block(&newobj);
        self.resources = self.resources.substract(&newobj.cost);
        self.buildings.push(newobj);
    }

    fn add_road(&mut self, coord: Coord) {
        self.map.set(coord.clone(), Tile::road());
        self.resources = self.resources.substract(&self.road_cost);
    }

    fn block(&mut self, newobj: &Building) {
        let Coord{x, y} = newobj.topleft;
        let Coord{x: dx, y: dy} = newobj.dimxy;
        for x in x..(x+dx) {
            for y in y..(y+dy) {
                self.map.set(Coord::new(x,y), Tile::wall());
            }
        }
    }

    fn get_building_to_build(&self, u: usize) -> Building {
        self.buildingtypes[u].clone().new_token(self.mousexy)
    }

    fn update_buildings(&mut self, i: i32) {
        for o in &self.buildings {
            (o.update_fn)(&mut self.resources, i);
        }
    }

    fn render_all(&self, tcod: &mut Tcod) {
        // go through all tiles, and set their background color
        for y in 0..self.mapxy.y {
            for x in 0..self.mapxy.x {
                let tile = self.map.get(&Coord::new(x,y)).unwrap();
                if tile.blocked {
                    tcod.con.set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
                } else {
                    tcod.con.set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
                }
                if tile.road {
                    tcod.con.set_default_foreground(GREY);
                    tcod.con.put_char(x, y, 178 as char, BackgroundFlag::None);
                }
            }
        }
        // draw all objects in the list
        for object in &self.buildings {
            object.draw(&mut tcod.con);
        }
    
        // draw object types at the bottom of the screen
        for (i, object) in self.buildingtypes.iter().enumerate() {
            let mut o = object.clone();
            o.topleft = Coord::new((i+1) as i32*4, self.mapxy.y + 1);
            o.draw(&mut tcod.root);
        }
    
        // currently selected buildingtype to draw
        let mut costs = Resources(Vec::new());
        if let Draw::Building(u) = self.draw {
            let mut current = self.buildingtypes[u].clone();
            // draw current at bottom-right as a reminder of current selection
            current.topleft = Coord::new(self.mapxy.x-5, self.mapxy.y+1);
            current.draw(&mut tcod.root);
            current.topleft = self.mousexy.clone();
            // draw current on the map as a preview
            current.draw(&mut tcod.con);
            costs = current.cost;
        }
        if let Draw::Road(r) = self.draw {
            costs = self.road_cost.clone();
            match r {
                Some(road_start) => {
                    match path::find_route(&self.map, road_start, self.mousexy) {
                        Some(path) => {
                            tcod.con.set_default_foreground(WHITE);
                            for c in path.iter() {        
                                tcod.con.put_char(c.x, c.y, 178 as char, BackgroundFlag::None);
                            }
                        },
                        None => {},
                    }
                },
                None => {
                    tcod.con.set_default_foreground(WHITE);
                    tcod.con.put_char(self.mousexy.x, self.mousexy.y, 178 as char, BackgroundFlag::None);
                },
            }
        }
    
        // draw messages
        tcod.con.set_default_foreground(WHITE); 
        // TODO this should be more informed by the particular game
        for (i, r) in (0..4).enumerate() {
            if r >= self.resources.len() {
                continue
            }
            let c = match r {
                0 => 15 as char,  // Money
                1 => 240 as char, // Wood
                2 => 209 as char, // Tools
                3 => 219 as char, // Stone
                _ => panic!(),
            };
            if self.draw == Draw::None || costs.get(r as usize) == 0 {
                tcod.con.print_rect(i as i32 * 10 + 1, 1, 100, 0, format!("{}{}", c, self.resources.get(r as usize)));
            } else {
                tcod.con.print_rect(i as i32 * 10 + 1, 1, 100, 0, format!("{}{}({})", c, self.resources.get(r as usize), costs.get(r as usize)));
            }
        }
        
        blit(&tcod.con, (0,0), (self.mapxy.x, self.mapxy.y), &mut tcod.root, (0,0), 1.0, 1.0); 
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    blocked: bool,
    road:    bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            road:    false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            road:    false,
        }
    }

    pub fn road() -> Self {
        Tile {
            blocked: true,
            road:    true,
        }
    }
}

fn von_neumann_neighbours(p: Coord) -> [Coord; 4] {
    [ Coord::new(p.x + 1, p.y),
      Coord::new(p.x - 1, p.y),
      Coord::new(p.x, p.y + 1),
      Coord::new(p.x, p.y - 1) ]
}

struct Map<'a> {
    coords: &'a mut HashMap<Coord,Tile>
}

impl<'a> path::Map for &Map<'a> {
    type Node = Coord; 
    fn neighbours(&self, n: Self::Node) -> Vec<Self::Node> {
        let mut neighbours: Vec<Self::Node> = vec![];
        for p in von_neumann_neighbours(n).iter() {
            let t = self.coords.get(&p);
            if let Some(tile) = t {
                if !tile.blocked || tile.road {
                    neighbours.push(*p);
                }
            }
        }
        neighbours
    }

    fn g(&self, _n: Self::Node, _neighbour: Self::Node) -> i64 { 1 }

    fn h(&self, n: Self::Node, goal: Self::Node) -> i64 {
        let dx = goal.x - n.x;
        let dy = goal.y - n.y;
        (dx.abs() + dy.abs()) as i64
    }
}

impl<'a> Map<'a> {
    fn new(m: &'a mut HashMap<Coord,Tile>) -> Self {
        Map{ coords: m }
    }

    fn get(&self, c: &Coord) -> Option<&Tile> {
        self.coords.get(c)
    }

    fn set(&mut self, c: Coord, t: Tile) {
        self.coords.insert(c, t);
    }
}

pub fn make_map(dimxy: Coord) -> HashMap<Coord, Tile> {
    let mut m = HashMap::new();
    for y in 0..dimxy.y {
        for x in 0..dimxy.x {
            if x > (2*dimxy.x/3) {
                m.insert(Coord::new(x,y), Tile::wall());
            } else {
                m.insert(Coord::new(x,y), Tile::empty());
            }
        }
    }
    m
}

