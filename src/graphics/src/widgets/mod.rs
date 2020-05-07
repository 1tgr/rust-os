use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;
use graphics_base::system::System;
use graphics_base::Result;
use hecs::{DynamicBundle, World};

pub trait WidgetSystem {
    type Widget: 'static;
    type Components: Clone + DynamicBundle;

    fn components(&self) -> Self::Components;

    fn run(&mut self, world: &mut World) -> Result<()> {
        struct WidgetRegistered<U>(PhantomData<U>);

        let entities = world
            .query::<()>()
            .with::<Self::Widget>()
            .without::<WidgetRegistered<Self::Widget>>()
            .iter()
            .collect::<Vec<_>>();

        if !entities.is_empty() {
            let components = self.components();
            for (entity, ()) in entities {
                let r: WidgetRegistered<Self::Widget> = WidgetRegistered(PhantomData);
                world.insert(entity, components.clone()).unwrap();
                world.insert_one(entity, r).unwrap();
            }
        }

        Ok(())
    }
}

mod button;
mod label;
mod text_box;

pub use button::Button;
pub use label::Label;
pub use text_box::TextBox;

pub struct ClientPortal;

pub(crate) fn register(systems: &mut Vec<Box<dyn System>>) {
    systems.push(Box::new(button::ButtonSystem::new()));
    systems.push(Box::new(label::LabelSystem::new()));
    systems.push(Box::new(text_box::TextBoxSystem::new()));
}
