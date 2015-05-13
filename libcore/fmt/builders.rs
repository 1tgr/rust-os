// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use prelude::*;
use fmt::{self, Write, FlagV1};

struct PadAdapter<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    on_newline: bool,
}

impl<'a, 'b: 'a> PadAdapter<'a, 'b> {
    fn new(fmt: &'a mut fmt::Formatter<'b>) -> PadAdapter<'a, 'b> {
        PadAdapter {
            fmt: fmt,
            on_newline: false,
        }
    }
}

impl<'a, 'b: 'a> fmt::Write for PadAdapter<'a, 'b> {
    fn write_str(&mut self, mut s: &str) -> fmt::Result {
        while !s.is_empty() {
            if self.on_newline {
                try!(self.fmt.write_str("    "));
            }

            let split = match s.find('\n') {
                Some(pos) => {
                    self.on_newline = true;
                    pos + 1
                }
                None => {
                    self.on_newline = false;
                    s.len()
                }
            };
            try!(self.fmt.write_str(&s[..split]));
            s = &s[split..];
        }

        Ok(())
    }
}

/// A struct to help with `fmt::Debug` implementations.
///
/// Constructed by the `Formatter::debug_struct` method.
#[must_use]
pub struct DebugStruct<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    result: fmt::Result,
    has_fields: bool,
}

pub fn debug_struct_new<'a, 'b>(fmt: &'a mut fmt::Formatter<'b>, name: &str)
                                -> DebugStruct<'a, 'b> {
    let result = fmt.write_str(name);
    DebugStruct {
        fmt: fmt,
        result: result,
        has_fields: false,
    }
}

impl<'a, 'b: 'a> DebugStruct<'a, 'b> {
    /// Adds a new field to the generated struct output.
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    pub fn field(mut self, name: &str, value: &fmt::Debug) -> DebugStruct<'a, 'b> {
        self.result = self.result.and_then(|_| {
            let prefix = if self.has_fields {
                ","
            } else {
                " {"
            };

            if self.is_pretty() {
                let mut writer = PadAdapter::new(self.fmt);
                fmt::write(&mut writer, format_args!("{}\n{}: {:#?}", prefix, name, value))
            } else {
                write!(self.fmt, "{} {}: {:?}", prefix, name, value)
            }
        });

        self.has_fields = true;
        self
    }

    /// Consumes the `DebugStruct`, finishing output and returning any error
    /// encountered.
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    pub fn finish(mut self) -> fmt::Result {
        if self.has_fields {
            self.result = self.result.and_then(|_| {
                if self.is_pretty() {
                    self.fmt.write_str("\n}")
                } else {
                    self.fmt.write_str(" }")
                }
            });
        }
        self.result
    }

    fn is_pretty(&self) -> bool {
        self.fmt.flags() & (1 << (FlagV1::Alternate as usize)) != 0
    }
}

/// A struct to help with `fmt::Debug` implementations.
///
/// Constructed by the `Formatter::debug_tuple` method.
#[must_use]
pub struct DebugTuple<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    result: fmt::Result,
    has_fields: bool,
}

pub fn debug_tuple_new<'a, 'b>(fmt: &'a mut fmt::Formatter<'b>, name: &str) -> DebugTuple<'a, 'b> {
    let result = fmt.write_str(name);
    DebugTuple {
        fmt: fmt,
        result: result,
        has_fields: false,
    }
}

impl<'a, 'b: 'a> DebugTuple<'a, 'b> {
    /// Adds a new field to the generated tuple struct output.
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    pub fn field(mut self, value: &fmt::Debug) -> DebugTuple<'a, 'b> {
        self.result = self.result.and_then(|_| {
            let (prefix, space) = if self.has_fields {
                (",", " ")
            } else {
                ("(", "")
            };

            if self.is_pretty() {
                let mut writer = PadAdapter::new(self.fmt);
                fmt::write(&mut writer, format_args!("{}\n{:#?}", prefix, value))
            } else {
                write!(self.fmt, "{}{}{:?}", prefix, space, value)
            }
        });

        self.has_fields = true;
        self
    }

    /// Consumes the `DebugTuple`, finishing output and returning any error
    /// encountered.
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    pub fn finish(mut self) -> fmt::Result {
        if self.has_fields {
            self.result = self.result.and_then(|_| {
                if self.is_pretty() {
                    self.fmt.write_str("\n)")
                } else {
                    self.fmt.write_str(")")
                }
            });
        }
        self.result
    }

    fn is_pretty(&self) -> bool {
        self.fmt.flags() & (1 << (FlagV1::Alternate as usize)) != 0
    }
}

struct DebugInner<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    result: fmt::Result,
    has_fields: bool,
}

impl<'a, 'b: 'a> DebugInner<'a, 'b> {
    fn entry(&mut self, entry: &fmt::Debug) {
        self.result = self.result.and_then(|_| {
            if self.is_pretty() {
                let mut writer = PadAdapter::new(self.fmt);
                let prefix = if self.has_fields { "," } else { "" };
                fmt::write(&mut writer, format_args!("{}\n{:#?}", prefix, entry))
            } else {
                let prefix = if self.has_fields { ", " } else { "" };
                write!(self.fmt, "{}{:?}", prefix, entry)
            }
        });

        self.has_fields = true;
    }

    pub fn finish(&mut self) {
        let prefix = if self.is_pretty() && self.has_fields { "\n" } else { "" };
        self.result = self.result.and_then(|_| self.fmt.write_str(prefix));
    }

    fn is_pretty(&self) -> bool {
        self.fmt.flags() & (1 << (FlagV1::Alternate as usize)) != 0
    }
}

/// A struct to help with `fmt::Debug` implementations.
///
/// Constructed by the `Formatter::debug_set` method.
#[must_use]
pub struct DebugSet<'a, 'b: 'a> {
    inner: DebugInner<'a, 'b>,
}

pub fn debug_set_new<'a, 'b>(fmt: &'a mut fmt::Formatter<'b>) -> DebugSet<'a, 'b> {
    let result = write!(fmt, "{{");
    DebugSet {
        inner: DebugInner {
            fmt: fmt,
            result: result,
            has_fields: false,
        }
    }
}

impl<'a, 'b: 'a> DebugSet<'a, 'b> {
    /// Adds a new entry to the set output.
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    pub fn entry(mut self, entry: &fmt::Debug) -> DebugSet<'a, 'b> {
        self.inner.entry(entry);
        self
    }

    /// Consumes the `DebugSet`, finishing output and returning any error
    /// encountered.
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    pub fn finish(mut self) -> fmt::Result {
        self.inner.finish();
        self.inner.result.and_then(|_| self.inner.fmt.write_str("}"))
    }
}

/// A struct to help with `fmt::Debug` implementations.
///
/// Constructed by the `Formatter::debug_list` method.
#[must_use]
pub struct DebugList<'a, 'b: 'a> {
    inner: DebugInner<'a, 'b>,
}

pub fn debug_list_new<'a, 'b>(fmt: &'a mut fmt::Formatter<'b>) -> DebugList<'a, 'b> {
    let result = write!(fmt, "[");
    DebugList {
        inner: DebugInner {
            fmt: fmt,
            result: result,
            has_fields: false,
        }
    }
}

impl<'a, 'b: 'a> DebugList<'a, 'b> {
    /// Adds a new entry to the set output.
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    pub fn entry(mut self, entry: &fmt::Debug) -> DebugList<'a, 'b> {
        self.inner.entry(entry);
        self
    }

    /// Consumes the `DebugSet`, finishing output and returning any error
    /// encountered.
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    pub fn finish(mut self) -> fmt::Result {
        self.inner.finish();
        self.inner.result.and_then(|_| self.inner.fmt.write_str("]"))
    }
}

/// A struct to help with `fmt::Debug` implementations.
///
/// Constructed by the `Formatter::debug_map` method.
#[must_use]
pub struct DebugMap<'a, 'b: 'a> {
    fmt: &'a mut fmt::Formatter<'b>,
    result: fmt::Result,
    has_fields: bool,
}

pub fn debug_map_new<'a, 'b>(fmt: &'a mut fmt::Formatter<'b>) -> DebugMap<'a, 'b> {
    let result = write!(fmt, "{{");
    DebugMap {
        fmt: fmt,
        result: result,
        has_fields: false,
    }
}

impl<'a, 'b: 'a> DebugMap<'a, 'b> {
    /// Adds a new entry to the map output.
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    pub fn entry(mut self, key: &fmt::Debug, value: &fmt::Debug) -> DebugMap<'a, 'b> {
        self.result = self.result.and_then(|_| {
            if self.is_pretty() {
                let mut writer = PadAdapter::new(self.fmt);
                let prefix = if self.has_fields { "," } else { "" };
                fmt::write(&mut writer, format_args!("{}\n{:#?}: {:#?}", prefix, key, value))
            } else {
                let prefix = if self.has_fields { ", " } else { "" };
                write!(self.fmt, "{}{:?}: {:?}", prefix, key, value)
            }
        });

        self.has_fields = true;
        self
    }

    /// Consumes the `DebugMap`, finishing output and returning any error
    /// encountered.
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    pub fn finish(self) -> fmt::Result {
        let prefix = if self.is_pretty() && self.has_fields { "\n" } else { "" };
        self.result.and_then(|_| write!(self.fmt, "{}}}", prefix))
    }

    fn is_pretty(&self) -> bool {
        self.fmt.flags() & (1 << (FlagV1::Alternate as usize)) != 0
    }
}
