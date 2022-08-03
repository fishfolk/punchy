use std::marker::PhantomData;

use bevy::{
    ecs::{
        entity::Entities,
        system::{Command, CommandQueue, EntityCommands, SystemParam},
    },
    prelude::*,
};

/// Resource containing the state transition command queue
pub struct CustomCommandQueue<Marker> {
    queue: CommandQueue,
    _phantom: PhantomData<Marker>,
}

impl<Marker> Default for CustomCommandQueue<Marker> {
    fn default() -> Self {
        Self {
            queue: Default::default(),
            _phantom: Default::default(),
        }
    }
}

/// System parameter very similar to the [`Commands`] parameter, but commands issued though it will
/// be flushed during [`FighterStateSystemSet::FlushStateTransitions`] instead of at the end of the
/// frame.
#[derive(SystemParam)]
pub struct CustomCommands<'w, 's, Marker: Sync + Send + 'static> {
    queue: ResMut<'w, CustomCommandQueue<Marker>>,
    entities: &'w Entities,
    #[system_param(ignore)]
    _phantom: PhantomData<(&'s (), Marker)>,
}

impl<'w, 's, Marker: Sync + Send + 'static> CustomCommands<'w, 's, Marker> {
    pub fn commands<'a>(&'a mut self) -> Commands<'w, 'a> {
        Commands::new_from_entities(&mut self.queue.queue, self.entities)
    }
}

/// Extension trait for [`App`] that adds a function to initialize a custom command queue.
pub trait InitCustomCommandsAppExt {
    /// Initialize a custom command queue tha will be usable as a system param with
    /// `CustomCommands<Marker>`.
    fn init_custom_commands<Marker: Sync + Send + 'static>(&mut self) -> &mut Self;
}

impl InitCustomCommandsAppExt for App {
    fn init_custom_commands<Marker: Sync + Send + 'static>(&mut self) -> &mut Self {
        self.init_resource::<CustomCommandQueue<Marker>>()
    }
}

/// Exclusive system that can be added to flush a custom command queue.
pub fn flush_custom_commands<Marker: Send + Sync + 'static>(world: &mut World) {
    let mut queue = world
        .remove_resource::<CustomCommandQueue<Marker>>()
        .unwrap();

    queue.queue.apply(world);

    world.insert_resource(queue);
}

/// Extension trait for [`EntityCommands`] that allows inserting a dynamic component.
pub trait DynamicEntityCommandsExt {
    /// Insert a component onto the entity that is not known at compile time.
    fn insert_dynamic(
        &mut self,
        reflect_component: ReflectComponent,
        component_data: Box<dyn Reflect>,
    ) -> &mut Self;
}

struct InsertDynamicCommand(Entity, ReflectComponent, Box<dyn Reflect>);
impl Command for InsertDynamicCommand {
    fn write(self, world: &mut World) {
        self.1.insert(world, self.0, self.2.as_reflect());
    }
}

impl<'w, 's, 'a> DynamicEntityCommandsExt for EntityCommands<'w, 's, 'a> {
    fn insert_dynamic(
        &mut self,
        reflect_component: ReflectComponent,
        component_data: Box<dyn Reflect>,
    ) -> &mut Self {
        let entity = self.id();
        self.commands().add(InsertDynamicCommand(
            entity,
            reflect_component,
            component_data,
        ));

        self
    }
}
