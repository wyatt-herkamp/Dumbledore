#![allow(dead_code,clippy::from_over_into)]

pub mod archetypes;
pub mod component;
pub mod component_ref;
pub mod entities;
pub mod sets;
pub mod world;

#[cfg(feature = "dumbledore-macro")]
pub use dumbledore_macro::Bundle;

#[cfg(test)]
pub mod tests {
    use crate::archetypes::ComponentInfo;
    use crate::component::{Bundle, Component};
    use std::ptr::NonNull;
    use crate::world::World;

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

    pub struct Player {
        pub position: Position,
        pub health: Health,
    }

    impl Bundle for Player {
        fn into_component_ptrs(mut self) -> Box<[(ComponentInfo, NonNull<u8>)]>
            where
                Self: Sized,
        {
            let position = &mut self.position as *mut Position;
            let health = &mut self.health as *mut Health;
            Box::new([
                (
                    ComponentInfo::new::<Position>(),
                    NonNull::new(position as *mut u8).unwrap(),
                ),
                (
                    ComponentInfo::new::<Health>(),
                    NonNull::new(health as *mut u8).unwrap(),
                ),
            ])
        }

        fn component_info() -> Vec<ComponentInfo>
            where
                Self: Sized,
        {
            vec![
                ComponentInfo::new::<Position>(),
                ComponentInfo::new::<Health>(),
            ]
        }

        fn archetype_id() -> u32
            where
                Self: Sized,
        {
            0
        }
    }

    #[test]
    pub fn test() {
        let mut world = World::new(256);
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
                let entity = world.get_entities().get_location(i).unwrap();
                let index = entity.index;
                let option = player
                    .get_comp::<(Position, Health)>(index).unwrap();
            }
        }
        println!("Finished in: {}", now.elapsed().as_millis());
    }

    #[test]
    pub fn entities_realloc() {
        let mut world = World::new(256);
        world.add_archetype::<Player>(256);
        for i in 0..1024 {
            if !world.get_entities().entities_left() {
                world.increase_entities(Some(256)).unwrap();
            }
            if let Err(error) = world
                .add_entity(Player {
                    position: Position { x: 0.0, y: 0.0 },
                    health: Health {
                        health: 100.0,
                        food: 100.0,
                    },
                }) {
                match error {
                    crate::world::WorldError::TooManyEntitiesInArchetype => {
                        let option = world.take_archetype::<Player>().unwrap();
                        let archetype = option.resize(Some(256)).unwrap();
                        world.push_archetype::<Player>(archetype);
                    }
                    _ => {}
                }
            }
        }
        let player = world.get_archetype::<Player>().unwrap();

        for _ in 0..2048 {
            for i in 0..1024 {
                let entity = world.get_entities().get_location(i).unwrap();
                let index = entity.index;
                let option = player
                    .get_comp::<(Position, Health)>(index).unwrap();
            }
        }
    }

    #[test]
    pub fn random_delete_an_add() {
        let mut world = World::new(256);
        world.add_archetype::<Player>(256);
        for i in 0..1024 {
            if !world.get_entities().entities_left() {
                world.increase_entities(Some(256)).unwrap();
            }
            if let Err(error) = world
                .add_entity(Player {
                    position: Position { x: 0.0, y: 0.0 },
                    health: Health {
                        health: 100.0,
                        food: 100.0,
                    },
                }) {
                match error {
                    crate::world::WorldError::TooManyEntitiesInArchetype => {
                        let option = world.take_archetype::<Player>().unwrap();
                        let archetype = option.resize(Some(256)).unwrap();
                        world.push_archetype::<Player>(archetype);
                    }
                    _ => {}
                }
            }
        }
        for _ in 0..256 {
            let random1: u8 = rand::random();
            let entity = crate::entities::entity::Entity::from(random1 as u32);

            world.remove_entity(entity);
            let player = world.get_archetype::<Player>().unwrap();

            let (entity, id) = world.add_entity(Player {
                position: Position { x: 0.0, y: 0.0 },
                health: Health {
                    health: 100.0,
                    food: 50.0,
                },
            }).unwrap();

            assert_eq!(entity.id, random1 as u32);

            if player
                .get_comp::<(Health)>(id.index).is_err() {
                println!(" {:?}", id);
                println!(" {:?}", entity);
            };
        }
    }
}

