#![feature(link_args)]
#![feature(proc_macro_hygiene)]

extern crate alloc;
extern crate alloc_system;
extern crate rt;

use ui::db::Database;
use ui_server::app;
use ui_types::Result;

#[allow(unused_attributes)]
#[link_args = "-Ttext-segment 0x40000000"]
extern "C" {}

fn main() -> Result<()> {
    app::run(Database::new())
}
