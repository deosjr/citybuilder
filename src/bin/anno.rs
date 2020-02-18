use tcod::colors::*;

#[macro_use]
mod game;

resource_types!{Money, Wood, Tools, Stone}

building_types!{House, Church, Lumberjack, Fisherman, Market}

fn update_house(resources: &mut game::Resources, i: i32) {
    if i % 100 != 0 { 
        return 
    }
    resources.update(Resource::Money, 1);
}

fn update_lumberjack(resources: &mut game::Resources, i: i32) {
    if i % 100 != 0 { 
        return 
    }
    resources.update(Resource::Wood, 1);
}

fn update_empty(_: &mut game::Resources, _: i32) {}

fn main() {
    use Resource::*;
    use BuildingType::*;
    use game::Game;
    use game::Coord;
    use game::building::Building;

    // we can also macro this
    // macro everything and make a DSL :)
    let buildings = [
        Building::new_type(House, Coord::new(2,2), resources![(Money, 10), (Wood, 3)], ORANGE, update_house),
        Building::new_type(Church, Coord::new(1,2), resources![(Money, 100), (Wood, 5), (Tools, 3)], YELLOW, update_empty),
        Building::new_type(Lumberjack, Coord::new(2,2), resources![(Money, 10), (Tools, 2)], GREEN, update_lumberjack),
        Building::new_type(Fisherman, Coord::new(2,2), resources![(Money, 10), (Wood, 3)], BLUE, update_empty),
        Building::new_type(Market, Coord::new(3,4), resources![(Money, 200), (Wood, 5)], RED, update_empty),
    ];

    // NOTE: buildings should be in sync with enum BuildingType
    assert!(buildings.len() == NumBuildingTypes as usize);

    // actual size of the window
    const MAP_WIDTH: i32 = 80;
    const MAP_HEIGHT: i32 = 45;

    let road_cost = resources![(Money, 2)];
    let mut map = game::make_map(Coord::new(MAP_WIDTH, MAP_HEIGHT));
    let mut game = Game::new(&buildings, MAP_WIDTH, MAP_HEIGHT, NumResources, road_cost, &mut map);
    game.resources.set(Money, 1000);
    game.resources.set(Wood, 50);
    game.resources.set(Tools, 10);

    let tcod = game::get_tcod(&game);
    game::do_loop(tcod, &mut game);
}
