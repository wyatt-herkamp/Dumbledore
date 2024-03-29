#![allow(dead_code, clippy::from_over_into)]

pub mod archetypes;
pub mod component;
pub mod component_ref;
pub mod entities;
pub mod sets;
pub mod world;

#[cfg(feature = "dumbledore-macro")]
pub use dumbledore_macro::Bundle;
#[cfg(feature = "dumbledore-macro")]
pub use dumbledore_macro::Component;

#[cfg(test)]
pub mod tests {
    use crate::archetypes::ComponentInfo;
    use crate::component::{Bundle, Component};
    use crate::world::World;
    use dumbledore_macro::Component;
    use std::mem;

    #[derive(Debug, Clone, Component)]
    pub struct Position {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Debug, Clone, Component)]
    pub struct Health {
        pub health: f32,
        pub food: f32,
    }

    pub struct Player {
        pub position: Position,
        pub health: Health,
    }

    impl Bundle for Player {
        unsafe fn put_self(self, mut f: impl FnMut(*mut u8, ComponentInfo))
        where
            Self: Sized,
        {
            let mut position = self.position;
            f(
                (&mut position as *mut Position).cast(),
                ComponentInfo::new::<Position>(),
            );
            mem::forget(position);
            let mut health = self.health;
            f(
                (&mut health as *mut Health).cast(),
                ComponentInfo::new::<Health>(),
            );
            mem::forget(health);
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
        for _ in 0..255 {
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
                player.get_comp::<(Position, Health)>(index).unwrap();
            }
        }
        println!("Finished in: {}", now.elapsed().as_millis());
    }

    #[test]
    pub fn entities_realloc() {
        let mut world = World::new(256);
        world.add_archetype::<Player>(256);
        for _ in 0..1024 {
            if !world.get_entities().entities_left() {
                world.increase_entities(Some(256)).unwrap();
            }
            if let Err(error) = world.add_entity(Player {
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
                player.get_comp::<(Position, Health)>(index).unwrap();
            }
        }
    }

    #[test]
    pub fn random_delete_an_add() {
        let mut world = World::new(256);
        world.add_archetype::<Player>(256);
        for _ in 0..1024 {
            if !world.get_entities().entities_left() {
                world.increase_entities(Some(256)).unwrap();
            }
            if let Err(error) = world.add_entity(Player {
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

            let (entity, id) = world
                .add_entity(Player {
                    position: Position { x: 0.0, y: 0.0 },
                    health: Health {
                        health: 100.0,
                        food: 50.0,
                    },
                })
                .unwrap();

            assert_eq!(entity.id, random1 as u32);

            if player.get_comp::<Health>(id.index).is_err() {
                println!(" {:?}", id);
                println!(" {:?}", entity);
            };
        }
    }
}
