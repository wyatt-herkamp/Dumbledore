pub mod world;
pub mod archetypes;
pub mod entities;
pub mod component;
pub mod sets;
pub mod component_ref;

pub struct MyComponent{
    pub name: String,
}
#[test]
pub fn test(){

    let world = world::World::new();
    let entity = world.create_entity(MyComponent{ name: "".to_string() });
}