use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
};

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
