use crate::geometry::{ObjectSize, ParentPoint, ParentSpace, ParentTransform, ScreenPath, ScreenTransform};
use crate::id_map::IdMap;
use crate::path::Path;
use crate::property_map::{Property, PropertyMap};
use crate::widget::{InternDb, WidgetId, WidgetObject};
use alloc::rc::Rc;
use alloc::vec::Vec;
use euclid::Box2D;
use raqote::SolidSource;
use ui_types::types::ScreenSpace;

macro_rules! properties {
    ( $( [ $upper: ident, $lower: ident, $ty: ty ] ),* ) => {
        $(
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
        pub struct $upper;

        impl Property for $upper {
            type Value = $ty;
        }
        )*

        #[salsa::query_group(PropertyStorage)]
        pub trait PropertyDb: InternDb {
            #[salsa::input]
            fn properties(&self) -> Rc<PropertyMap>;

            $(
            fn $lower(&self, widget_id: WidgetId) -> Option<$ty>;
            )*

            fn parents(&self) -> Rc<IdMap<WidgetId, WidgetId>>;
            fn children(&self, widget_id: WidgetId) -> Vec<WidgetId>;
            fn screen_transform(&self, widget_id: WidgetId) -> ScreenTransform;

            #[salsa::transparent]
            fn screen_path(&self, widget_id: WidgetId) -> ScreenPath;
        }

        $(
        fn $lower(db: &dyn PropertyDb, widget_id: WidgetId) -> Option<$ty> {
            db.properties().get(widget_id, &$upper).cloned()
        }
        )*
    };
}

properties! {
    [ Archetype, archetype, Rc<dyn WidgetObject> ],
    [ Color, color, SolidSource ],
    [ Parent, parent, WidgetId ],
    [ Origin, origin, ParentPoint ],
    [ Size, size, ObjectSize ],
    [ Text, text, Rc<String> ],
    [ TextColor, text_color, SolidSource ],
    [ Transform, transform, ParentTransform ]
}

fn parents(db: &dyn PropertyDb) -> Rc<IdMap<WidgetId, WidgetId>> {
    Rc::new(
        db.properties()
            .iter(&Parent)
            .map(|(child_id, &parent_id)| (child_id, parent_id))
            .collect(),
    )
}

fn children(db: &dyn PropertyDb, widget_id: WidgetId) -> Vec<WidgetId> {
    let mut children = db
        .parents()
        .iter()
        .filter_map(|(child_id, parent_id)| widget_id.eq(parent_id).then(|| child_id))
        .collect::<Vec<_>>();

    children.sort();
    children
}

fn screen_transform(db: &dyn PropertyDb, widget_id: WidgetId) -> ScreenTransform {
    let transform = db.transform(widget_id).unwrap_or_default();
    db.parent(widget_id)
        .map_or(transform.with_destination::<ScreenSpace>(), |parent| {
            transform.then(&screen_transform(db, parent).with_source::<ParentSpace>())
        })
}

fn screen_path(db: &dyn PropertyDb, widget_id: WidgetId) -> ScreenPath {
    let size = db.size(widget_id).unwrap_or_default();
    let path = Path::Rect(Box2D::from_size(size));
    let transform = db.screen_transform(widget_id);
    path.transform(&transform)
}
