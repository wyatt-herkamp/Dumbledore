use std::ptr::NonNull;
use dumbledore::{Bundle, world};
use dumbledore::component::Component;

#[derive(Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct Health {
    pub health: f32,
    pub food: f32,
}

impl Component for Position {}

impl Component for Health {}
#[derive(Debug, Clone, Bundle)]
#[bundle(id = 0)]
pub struct Player {
    pub position: Position,
    pub health: Health,
}


fn main() {
    let mut world = world::World::new(256);
    world.add_archetype::<Player>(256);
    for i in 0..255 {
        world
            .add_entity(Player {
                position: Position { x: 0.0, y: 0.0 },
                health: Health {
                    health: 100.0,
                    food: 100.0,
                },
            })
            .unwrap();
    }
    let player = world.get_archetype::<Player>().unwrap();
    let now = std::time::Instant::now();

    for _ in 0..2048 {
        for i in 0..255 {
            let (entity, location) = world.get_entities().get_entity(i).unwrap();
            let index = location.index;
            let option = player
                .get_comp::<(Position, Health)>(index).unwrap();
        }
    }
    println!("Finished in: {}", now.elapsed().as_millis());
}