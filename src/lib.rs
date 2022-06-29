
pub mod world;
pub mod archetypes;
pub mod entities;
pub mod component;
pub mod sets;
pub mod component_ref;

#[cfg(test)]
pub mod tests {
    use std::ptr::NonNull;
    use crate::archetypes::ComponentInfo;
    use crate::component::{Bundle, Component};

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
    pub struct Player{
        pub position: Position,
        pub health: Health,
    }
    impl Bundle for Player {
        fn into_component_ptrs(mut self) -> Box<[(ComponentInfo, NonNull<u8>)]> where Self: Sized {
            let position = &mut self.position as *mut Position;
            let health = &mut self.health as *mut Health;
            Box::new([
                (ComponentInfo::new::<Position>(), NonNull::new(position as *mut u8).unwrap()),
                (ComponentInfo::new::<Health>(), NonNull::new(health as *mut u8).unwrap()),
            ])
        }

        fn component_info() -> Vec<ComponentInfo> where Self: Sized {
            vec![
                ComponentInfo::new::<Position>(),
                ComponentInfo::new::<Health>(),
            ]
        }

        fn archetype_id() -> u32 where Self: Sized {
            0
        }
    }
}

#[test]
pub fn test() {
    let mut world = world::World::new(256);
    world.add_archetype::<tests::Player>(256);
    let result = world.add_entity(tests::Player {
        position: tests::Position { x: 0.0, y: 0.0 },
        health: tests::Health { health: 100.0, food: 100.0 },
    }).unwrap();

    let player = world.archetypes.get(&0).unwrap();
    let option = player.get_comp_mut::<tests::Position>(result.id).unwrap().unwrap();
    println!("{:?}", option);

}