use crate::system::System;
use crate::Result;
use core::marker::PhantomData;
use hecs::{DynamicBundle, World};

pub trait Widget {
    type System: System + Default;
}

pub trait WidgetSystem {
    type Widget;
    type Components: Clone + DynamicBundle;

    fn components(&self) -> Self::Components;
}

impl<T> System for T
where
    T: WidgetSystem + 'static,
{
    fn run(&mut self, world: &mut World) -> Result<()> {
        struct WidgetRegistered<U>(PhantomData<U>);

        let entities = world
            .query::<()>()
            .with::<T::Widget>()
            .without::<WidgetRegistered<T::Widget>>()
            .iter()
            .collect::<Vec<_>>();

        if !entities.is_empty() {
            let components = self.components();
            for (entity, ()) in entities {
                let r: WidgetRegistered<T::Widget> = WidgetRegistered(PhantomData);
                world.insert(entity, components.clone()).unwrap();
                world.insert_one(entity, r).unwrap();
            }
        }

        Ok(())
    }
}

mod button;
mod label;

pub use button::Button;
pub use label::Label;
