#![feature(proc_macro_hygiene)]

extern crate alloc_system;
extern crate rt;

#[macro_use]
extern crate alloc;

use euclid::{Angle, Transform2D, Vector2D};
use raqote::SolidSource;
use ui::app::run;
use ui::db::Database;
use ui::input::InputDb;
use ui::property::PropertyDb;
use ui::widget::WidgetId;
use ui::Result;
use ui_macros::render;

fn color(db: &Database, widget_id: WidgetId) -> SolidSource {
    let pos = db.mouse_pos();
    if db.screen_path(widget_id).contains(0.0, pos) {
        SolidSource::from_unpremultiplied_argb(0x80, 0xff, 0, 0)
    } else {
        SolidSource::from_unpremultiplied_argb(0x80, 0, 0xff, 0)
    }
}

fn main() -> Result<()> {
    run(Database::new(), move |db| {
        render! {
            <portal
                origin={(100.0, 100.0)}
                size={(250.0, 250.0)}
                color={color(db, me)}
            >
                for index in 0..36 {
                    <panel
                        key={format!("a{}", index)}
                        transform={Transform2D::rotation(Angle::degrees(index as f32 * 5.0)).then_translate(Vector2D::new(20.0 + index as f32 * 20.0, 10.0 + index as f32 * 10.0))}
                        size={(50.0, 50.0)}
                        color={color(db, me)}
                    />
                }
                for index in 0..36 {
                    <panel
                        key={format!("b{}", index)}
                        transform={Transform2D::rotation(Angle::degrees(index as f32 * 5.0)).then_translate(Vector2D::new(20.0 + index as f32 * 20.0, 100.0 + index as f32 * 10.0))}
                        size={(50.0, 50.0)}
                        color={color(db, me)}
                    />
                }
                <button
                    transform={Transform2D::translation(10.0, 10.0)}
                    size={(100.0, 50.0)}
                    text={"Hello World".to_owned()}
                />
            </panel>
        }
    })
}
