pub mod archetypes;
pub mod component;
pub mod component_ref;
pub mod entities;
pub mod sets;
pub mod world;

#[cfg(test)]
pub mod tests {
    use crate::archetypes::ComponentInfo;
    use crate::component::{Bundle, Component};
    use std::ptr::NonNull;

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
}

#[test]
pub fn test() {
    let mut world = world::World::new(256);
    world.add_archetype::<tests::Player>(256);
    for i in 0..255 {
        world
            .add_entity(tests::Player {
                position: tests::Position { x: 0.0, y: 0.0 },
                health: tests::Health {
                    health: 100.0,
                    food: 100.0,
                },
            })
            .unwrap();
    }
    let player = world.archetypes.get(&0).unwrap();
    for _ in 0..2048 {
        for i in 0..255 {
            let entity = world.entities.get_location(i).unwrap();
            let index = entity.get_index();
            let option = player
                .get_comp_mut::<tests::Position>(index)
                .unwrap()
                .unwrap();
            let option = player
                .get_comp_mut::<tests::Health>(index)
                .unwrap()
                .unwrap();
        }
    }
    println!("{:?}", world.entities.get_location(0).unwrap());
}
