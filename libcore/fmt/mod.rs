// Copyright 2013-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities for formatting and printing strings

#![stable(feature = "rust1", since = "1.0.0")]

use prelude::*;

use cell::{Cell, RefCell, Ref, RefMut, BorrowState};
use marker::PhantomData;
use mem;
use ops::Deref;
use result;
use num::Float;
use slice;
use str;
use self::rt::v1::Alignment;

pub use self::num::radix;
pub use self::num::Radix;
pub use self::num::RadixFmt;

pub use self::builders::{DebugStruct, DebugTuple, DebugSet, DebugList, DebugMap};

mod num;
mod float;
mod builders;

#[unstable(feature = "core", reason = "internal to format_args!")]
#[doc(hidden)]
pub mod rt {
    pub mod v1;
}

#[stable(feature = "rust1", since = "1.0.0")]
/// The type returned by formatter methods.
pub type Result = result::Result<(), Error>;

/// The error type which is returned from formatting a message into a stream.
///
/// This type does not support transmission of an error other than that an error
/// occurred. Any extra information must be arranged to be transmitted through
/// some other means.
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Copy, Clone, Debug)]
pub struct Error;

/// A collection of methods that are required to format a message into a stream.
///
/// This trait is the type which this modules requires when formatting
/// information. This is similar to the standard library's `io::Write` trait,
/// but it is only intended for use in libcore.
///
/// This trait should generally not be implemented by consumers of the standard
/// library. The `write!` macro accepts an instance of `io::Write`, and the
/// `io::Write` trait is favored over implementing this trait.
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Write {
    /// Writes a slice of bytes into this writer, returning whether the write
    /// succeeded.
    ///
    /// This method can only succeed if the entire byte slice was successfully
    /// written, and this method will not return until all data has been
    /// written or an error occurs.
    ///
    /// # Errors
    ///
    /// This function will return an instance of `FormatError` on error.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn write_str(&mut self, s: &str) -> Result;

    /// Writes a `char` into this writer, returning whether the write succeeded.
    ///
    /// A single `char` may be encoded as more than one byte.
    /// This method can only succeed if the entire byte sequence was successfully
    /// written, and this method will not return until all data has been
    /// written or an error occurs.
    ///
    /// # Errors
    ///
    /// This function will return an instance of `FormatError` on error.
    #[stable(feature = "fmt_write_char", since = "1.1.0")]
    fn write_char(&mut self, c: char) -> Result {
        let mut utf_8 = [0u8; 4];
        let bytes_written = c.encode_utf8(&mut utf_8).unwrap_or(0);
        self.write_str(unsafe { mem::transmute(&utf_8[..bytes_written]) })
    }

    /// Glue for usage of the `write!` macro with implementers of this trait.
    ///
    /// This method should generally not be invoked manually, but rather through
    /// the `write!` macro itself.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn write_fmt(&mut self, args: Arguments) -> Result {
        // This Adapter is needed to allow `self` (of type `&mut
        // Self`) to be cast to a Write (below) without
        // requiring a `Sized` bound.
        struct Adapter<'a,T: ?Sized +'a>(&'a mut T);

        impl<'a, T: ?Sized> Write for Adapter<'a, T>
            where T: Write
        {
            fn write_str(&mut self, s: &str) -> Result {
                self.0.write_str(s)
            }

            fn write_fmt(&mut self, args: Arguments) -> Result {
                self.0.write_fmt(args)
            }
        }

        write(&mut Adapter(self), args)
    }
}

/// A struct to represent both where to emit formatting strings to and how they
/// should be formatted. A mutable version of this is passed to all formatting
/// traits.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Formatter<'a> {
    flags: u32,
    fill: char,
    align: rt::v1::Alignment,
    width: Option<usize>,
    precision: Option<usize>,

    buf: &'a mut (Write+'a),
    curarg: slice::Iter<'a, ArgumentV1<'a>>,
    args: &'a [ArgumentV1<'a>],
}

// NB. Argument is essentially an optimized partially applied formatting function,
// equivalent to `exists T.(&T, fn(&T, &mut Formatter) -> Result`.

enum Void {}

/// This struct represents the generic "argument" which is taken by the Xprintf
/// family of functions. It contains a function to format the given value. At
/// compile time it is ensured that the function and the value have the correct
/// types, and then this struct is used to canonicalize arguments to one type.
#[derive(Copy)]
#[unstable(feature = "core", reason = "internal to format_args!")]
#[doc(hidden)]
pub struct ArgumentV1<'a> {
    value: &'a Void,
    formatter: fn(&Void, &mut Formatter) -> Result,
}

impl<'a> Clone for ArgumentV1<'a> {
    fn clone(&self) -> ArgumentV1<'a> {
        *self
    }
}

impl<'a> ArgumentV1<'a> {
    #[inline(never)]
    fn show_usize(x: &usize, f: &mut Formatter) -> Result {
        Display::fmt(x, f)
    }

    #[doc(hidden)]
    #[unstable(feature = "core", reason = "internal to format_args!")]
    pub fn new<'b, T>(x: &'b T,
                      f: fn(&T, &mut Formatter) -> Result) -> ArgumentV1<'b> {
        unsafe {
            ArgumentV1 {
                formatter: mem::transmute(f),
                value: mem::transmute(x)
            }
        }
    }

    #[doc(hidden)]
    #[unstable(feature = "core", reason = "internal to format_args!")]
    pub fn from_usize(x: &usize) -> ArgumentV1 {
        ArgumentV1::new(x, ArgumentV1::show_usize)
    }

    fn as_usize(&self) -> Option<usize> {
        if self.formatter as usize == ArgumentV1::show_usize as usize {
            Some(unsafe { *(self.value as *const _ as *const usize) })
        } else {
            None
        }
    }
}

// flags available in the v1 format of format_args
#[derive(Copy, Clone)]
#[allow(dead_code)] // SignMinus isn't currently used
enum FlagV1 { SignPlus, SignMinus, Alternate, SignAwareZeroPad, }

impl<'a> Arguments<'a> {
    /// When using the format_args!() macro, this function is used to generate the
    /// Arguments structure.
    #[doc(hidden)] #[inline]
    #[unstable(feature = "core", reason = "internal to format_args!")]
    pub fn new_v1(pieces: &'a [&'a str],
                  args: &'a [ArgumentV1<'a>]) -> Arguments<'a> {
        Arguments {
            pieces: pieces,
            fmt: None,
            args: args
        }
    }

    /// This function is used to specify nonstandard formatting parameters.
    /// The `pieces` array must be at least as long as `fmt` to construct
    /// a valid Arguments structure. Also, any `Count` within `fmt` that is
    /// `CountIsParam` or `CountIsNextParam` has to point to an argument
    /// created with `argumentusize`. However, failing to do so doesn't cause
    /// unsafety, but will ignore invalid .
    #[doc(hidden)] #[inline]
    #[unstable(feature = "core", reason = "internal to format_args!")]
    pub fn new_v1_formatted(pieces: &'a [&'a str],
                            args: &'a [ArgumentV1<'a>],
                            fmt: &'a [rt::v1::Argument]) -> Arguments<'a> {
        Arguments {
            pieces: pieces,
            fmt: Some(fmt),
            args: args
        }
    }
}

/// This structure represents a safely precompiled version of a format string
/// and its arguments. This cannot be generated at runtime because it cannot
/// safely be done so, so no constructors are given and the fields are private
/// to prevent modification.
///
/// The `format_args!` macro will safely create an instance of this structure
/// and pass it to a function or closure, passed as the first argument. The
/// macro validates the format string at compile-time so usage of the `write`
/// and `format` functions can be safely performed.
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Copy, Clone)]
pub struct Arguments<'a> {
    // Format string pieces to print.
    pieces: &'a [&'a str],

    // Placeholder specs, or `None` if all specs are default (as in "{}{}").
    fmt: Option<&'a [rt::v1::Argument]>,

    // Dynamic arguments for interpolation, to be interleaved with string
    // pieces. (Every argument is preceded by a string piece.)
    args: &'a [ArgumentV1<'a>],
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Debug for Arguments<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        Display::fmt(self, fmt)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Display for Arguments<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        write(fmt.buf, *self)
    }
}

/// Format trait for the `:?` format. Useful for debugging, all types
/// should implement this.
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_on_unimplemented = "`{Self}` cannot be formatted using `:?`; if it is \
                            defined in your crate, add `#[derive(Debug)]` or \
                            manually implement it"]
#[lang = "debug_trait"]
pub trait Debug {
    /// Formats the value using the given formatter.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn fmt(&self, &mut Formatter) -> Result;
}

/// When a value can be semantically expressed as a String, this trait may be
/// used. It corresponds to the default format, `{}`.
#[rustc_on_unimplemented = "`{Self}` cannot be formatted with the default \
                            formatter; try using `:?` instead if you are using \
                            a format string"]
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Display {
    /// Formats the value using the given formatter.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn fmt(&self, &mut Formatter) -> Result;
}

/// Format trait for the `o` character
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Octal {
    /// Formats the value using the given formatter.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn fmt(&self, &mut Formatter) -> Result;
}

/// Format trait for the `b` character
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Binary {
    /// Formats the value using the given formatter.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn fmt(&self, &mut Formatter) -> Result;
}

/// Format trait for the `x` character
#[stable(feature = "rust1", since = "1.0.0")]
pub trait LowerHex {
    /// Formats the value using the given formatter.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn fmt(&self, &mut Formatter) -> Result;
}

/// Format trait for the `X` character
#[stable(feature = "rust1", since = "1.0.0")]
pub trait UpperHex {
    /// Formats the value using the given formatter.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn fmt(&self, &mut Formatter) -> Result;
}

/// Format trait for the `p` character
#[stable(feature = "rust1", since = "1.0.0")]
pub trait Pointer {
    /// Formats the value using the given formatter.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn fmt(&self, &mut Formatter) -> Result;
}

/// Format trait for the `e` character
#[stable(feature = "rust1", since = "1.0.0")]
pub trait LowerExp {
    /// Formats the value using the given formatter.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn fmt(&self, &mut Formatter) -> Result;
}

/// Format trait for the `E` character
#[stable(feature = "rust1", since = "1.0.0")]
pub trait UpperExp {
    /// Formats the value using the given formatter.
    #[stable(feature = "rust1", since = "1.0.0")]
    fn fmt(&self, &mut Formatter) -> Result;
}

/// The `write` function takes an output stream, a precompiled format string,
/// and a list of arguments. The arguments will be formatted according to the
/// specified format string into the output stream provided.
///
/// # Arguments
///
///   * output - the buffer to write output to
///   * args - the precompiled arguments generated by `format_args!`
#[stable(feature = "rust1", since = "1.0.0")]
pub fn write(output: &mut Write, args: Arguments) -> Result {
    let mut formatter = Formatter {
        flags: 0,
        width: None,
        precision: None,
        buf: output,
        align: Alignment::Unknown,
        fill: ' ',
        args: args.args,
        curarg: args.args.iter(),
    };

    let mut pieces = args.pieces.iter();

    match args.fmt {
        None => {
            // We can use default formatting parameters for all arguments.
            for (arg, piece) in args.args.iter().zip(pieces.by_ref()) {
                try!(formatter.buf.write_str(*piece));
                try!((arg.formatter)(arg.value, &mut formatter));
            }
        }
        Some(fmt) => {
            // Every spec has a corresponding argument that is preceded by
            // a string piece.
            for (arg, piece) in fmt.iter().zip(pieces.by_ref()) {
                try!(formatter.buf.write_str(*piece));
                try!(formatter.run(arg));
            }
        }
    }

    // There can be only one trailing string piece left.
    match pieces.next() {
        Some(piece) => {
            try!(formatter.buf.write_str(*piece));
        }
        None => {}
    }

    Ok(())
}

impl<'a> Formatter<'a> {

    // First up is the collection of functions used to execute a format string
    // at runtime. This consumes all of the compile-time statics generated by
    // the format! syntax extension.
    fn run(&mut self, arg: &rt::v1::Argument) -> Result {
        // Fill in the format parameters into the formatter
        self.fill = arg.format.fill;
        self.align = arg.format.align;
        self.flags = arg.format.flags;
        self.width = self.getcount(&arg.format.width);
        self.precision = self.getcount(&arg.format.precision);

        // Extract the correct argument
        let value = match arg.position {
            rt::v1::Position::Next => { *self.curarg.next().unwrap() }
            rt::v1::Position::At(i) => self.args[i],
        };

        // Then actually do some printing
        (value.formatter)(value.value, self)
    }

    fn getcount(&mut self, cnt: &rt::v1::Count) -> Option<usize> {
        match *cnt {
            rt::v1::Count::Is(n) => Some(n),
            rt::v1::Count::Implied => None,
            rt::v1::Count::Param(i) => {
                self.args[i].as_usize()
            }
            rt::v1::Count::NextParam => {
                self.curarg.next().and_then(|arg| arg.as_usize())
            }
        }
    }

    // Helper methods used for padding and processing formatting arguments that
    // all formatting traits can use.

    /// Performs the correct padding for an integer which has already been
    /// emitted into a str. The str should *not* contain the sign for the
    /// integer, that will be added by this method.
    ///
    /// # Arguments
    ///
    /// * is_positive - whether the original integer was positive or not.
    /// * prefix - if the '#' character (Alternate) is provided, this
    ///   is the prefix to put in front of the number.
    /// * buf - the byte array that the number has been formatted into
    ///
    /// This function will correctly account for the flags provided as well as
    /// the minimum width. It will not take precision into account.
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn pad_integral(&mut self,
                        is_positive: bool,
                        prefix: &str,
                        buf: &str)
                        -> Result {
        use char::CharExt;

        let mut width = buf.len();

        let mut sign = None;
        if !is_positive {
            sign = Some('-'); width += 1;
        } else if self.flags & (1 << (FlagV1::SignPlus as u32)) != 0 {
            sign = Some('+'); width += 1;
        }

        let mut prefixed = false;
        if self.flags & (1 << (FlagV1::Alternate as u32)) != 0 {
            prefixed = true; width += prefix.char_len();
        }

        // Writes the sign if it exists, and then the prefix if it was requested
        let write_prefix = |f: &mut Formatter| {
            if let Some(c) = sign {
                let mut b = [0; 4];
                let n = c.encode_utf8(&mut b).unwrap_or(0);
                let b = unsafe { str::from_utf8_unchecked(&b[..n]) };
                try!(f.buf.write_str(b));
            }
            if prefixed { f.buf.write_str(prefix) }
            else { Ok(()) }
        };

        // The `width` field is more of a `min-width` parameter at this point.
        match self.width {
            // If there's no minimum length requirements then we can just
            // write the bytes.
            None => {
                try!(write_prefix(self)); self.buf.write_str(buf)
            }
            // Check if we're over the minimum width, if so then we can also
            // just write the bytes.
            Some(min) if width >= min => {
                try!(write_prefix(self)); self.buf.write_str(buf)
            }
            // The sign and prefix goes before the padding if the fill character
            // is zero
            Some(min) if self.flags & (1 << (FlagV1::SignAwareZeroPad as u32)) != 0 => {
                self.fill = '0';
                try!(write_prefix(self));
                self.with_padding(min - width, Alignment::Right, |f| {
                    f.buf.write_str(buf)
                })
            }
            // Otherwise, the sign and prefix goes after the padding
            Some(min) => {
                self.with_padding(min - width, Alignment::Right, |f| {
                    try!(write_prefix(f)); f.buf.write_str(buf)
                })
            }
        }
    }

    /// This function takes a string slice and emits it to the internal buffer
    /// after applying the relevant formatting flags specified. The flags
    /// recognized for generic strings are:
    ///
    /// * width - the minimum width of what to emit
    /// * fill/align - what to emit and where to emit it if the string
    ///                provided needs to be padded
    /// * precision - the maximum length to emit, the string is truncated if it
    ///               is longer than this length
    ///
    /// Notably this function ignored the `flag` parameters
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn pad(&mut self, s: &str) -> Result {
        // Make sure there's a fast path up front
        if self.width.is_none() && self.precision.is_none() {
            return self.buf.write_str(s);
        }
        // The `precision` field can be interpreted as a `max-width` for the
        // string being formatted
        match self.precision {
            Some(max) => {
                // If there's a maximum width and our string is longer than
                // that, then we must always have truncation. This is the only
                // case where the maximum length will matter.
                let char_len = s.char_len();
                if char_len >= max {
                    let nchars = ::cmp::min(max, char_len);
                    return self.buf.write_str(s.slice_chars(0, nchars));
                }
            }
            None => {}
        }
        // The `width` field is more of a `min-width` parameter at this point.
        match self.width {
            // If we're under the maximum length, and there's no minimum length
            // requirements, then we can just emit the string
            None => self.buf.write_str(s),
            // If we're under the maximum width, check if we're over the minimum
            // width, if so it's as easy as just emitting the string.
            Some(width) if s.char_len() >= width => {
                self.buf.write_str(s)
            }
            // If we're under both the maximum and the minimum width, then fill
            // up the minimum width with the specified string + some alignment.
            Some(width) => {
                self.with_padding(width - s.char_len(), Alignment::Left, |me| {
                    me.buf.write_str(s)
                })
            }
        }
    }

    /// Runs a callback, emitting the correct padding either before or
    /// afterwards depending on whether right or left alignment is requested.
    fn with_padding<F>(&mut self, padding: usize, default: Alignment,
                       f: F) -> Result
        where F: FnOnce(&mut Formatter) -> Result,
    {
        use char::CharExt;
        let align = match self.align {
            Alignment::Unknown => default,
            _ => self.align
        };

        let (pre_pad, post_pad) = match align {
            Alignment::Left => (0, padding),
            Alignment::Right | Alignment::Unknown => (padding, 0),
            Alignment::Center => (padding / 2, (padding + 1) / 2),
        };

        let mut fill = [0; 4];
        let len = self.fill.encode_utf8(&mut fill).unwrap_or(0);
        let fill = unsafe { str::from_utf8_unchecked(&fill[..len]) };

        for _ in 0..pre_pad {
            try!(self.buf.write_str(fill));
        }

        try!(f(self));

        for _ in 0..post_pad {
            try!(self.buf.write_str(fill));
        }

        Ok(())
    }

    /// Writes some data to the underlying buffer contained within this
    /// formatter.
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn write_str(&mut self, data: &str) -> Result {
        self.buf.write_str(data)
    }

    /// Writes some formatted information into this instance
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn write_fmt(&mut self, fmt: Arguments) -> Result {
        write(self.buf, fmt)
    }

    /// Flags for formatting (packed version of rt::Flag)
    #[stable(feature = "rust1", since = "1.0.0")]
    pub fn flags(&self) -> u32 { self.flags }

    /// Character used as 'fill' whenever there is alignment
    #[unstable(feature = "core", reason = "method was just created")]
    pub fn fill(&self) -> char { self.fill }

    /// Flag indicating what form of alignment was requested
    #[unstable(feature = "core", reason = "method was just created")]
    pub fn align(&self) -> Alignment { self.align }

    /// Optionally specified integer width that the output should be
    #[unstable(feature = "core", reason = "method was just created")]
    pub fn width(&self) -> Option<usize> { self.width }

    /// Optionally specified precision for numeric types
    #[unstable(feature = "core", reason = "method was just created")]
    pub fn precision(&self) -> Option<usize> { self.precision }

    /// Creates a `DebugStruct` builder designed to assist with creation of
    /// `fmt::Debug` implementations for structs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(debug_builders, core)]
    /// use std::fmt;
    ///
    /// struct Foo {
    ///     bar: i32,
    ///     baz: String,
    /// }
    ///
    /// impl fmt::Debug for Foo {
    ///     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    ///         fmt.debug_struct("Foo")
    ///             .field("bar", &self.bar)
    ///             .field("baz", &self.baz)
    ///             .finish()
    ///     }
    /// }
    ///
    /// // prints "Foo { bar: 10, baz: "Hello World" }"
    /// println!("{:?}", Foo { bar: 10, baz: "Hello World".to_string() });
    /// ```
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    #[inline]
    pub fn debug_struct<'b>(&'b mut self, name: &str) -> DebugStruct<'b, 'a> {
        builders::debug_struct_new(self, name)
    }

    /// Creates a `DebugTuple` builder designed to assist with creation of
    /// `fmt::Debug` implementations for tuple structs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(debug_builders, core)]
    /// use std::fmt;
    ///
    /// struct Foo(i32, String);
    ///
    /// impl fmt::Debug for Foo {
    ///     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    ///         fmt.debug_tuple("Foo")
    ///             .field(&self.0)
    ///             .field(&self.1)
    ///             .finish()
    ///     }
    /// }
    ///
    /// // prints "Foo(10, "Hello World")"
    /// println!("{:?}", Foo(10, "Hello World".to_string()));
    /// ```
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    #[inline]
    pub fn debug_tuple<'b>(&'b mut self, name: &str) -> DebugTuple<'b, 'a> {
        builders::debug_tuple_new(self, name)
    }

    /// Creates a `DebugList` builder designed to assist with creation of
    /// `fmt::Debug` implementations for list-like structures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(debug_builders, core)]
    /// use std::fmt;
    ///
    /// struct Foo(Vec<i32>);
    ///
    /// impl fmt::Debug for Foo {
    ///     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    ///         self.0.iter().fold(fmt.debug_list(), |b, e| b.entry(e)).finish()
    ///     }
    /// }
    ///
    /// // prints "[10, 11]"
    /// println!("{:?}", Foo(vec![10, 11]));
    /// ```
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    #[inline]
    pub fn debug_list<'b>(&'b mut self) -> DebugList<'b, 'a> {
        builders::debug_list_new(self)
    }

    /// Creates a `DebugSet` builder designed to assist with creation of
    /// `fmt::Debug` implementations for set-like structures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(debug_builders, core)]
    /// use std::fmt;
    ///
    /// struct Foo(Vec<i32>);
    ///
    /// impl fmt::Debug for Foo {
    ///     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    ///         self.0.iter().fold(fmt.debug_set(), |b, e| b.entry(e)).finish()
    ///     }
    /// }
    ///
    /// // prints "{10, 11}"
    /// println!("{:?}", Foo(vec![10, 11]));
    /// ```
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    #[inline]
    pub fn debug_set<'b>(&'b mut self) -> DebugSet<'b, 'a> {
        builders::debug_set_new(self)
    }

    /// Creates a `DebugMap` builder designed to assist with creation of
    /// `fmt::Debug` implementations for map-like structures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(debug_builders, core)]
    /// use std::fmt;
    ///
    /// struct Foo(Vec<(String, i32)>);
    ///
    /// impl fmt::Debug for Foo {
    ///     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    ///         self.0.iter().fold(fmt.debug_map(), |b, &(ref k, ref v)| b.entry(k, v)).finish()
    ///     }
    /// }
    ///
    /// // prints "{"A": 10, "B": 11}"
    /// println!("{:?}", Foo(vec![("A".to_string(), 10), ("B".to_string(), 11)]));
    /// ```
    #[unstable(feature = "debug_builders", reason = "method was just created")]
    #[inline]
    pub fn debug_map<'b>(&'b mut self) -> DebugMap<'b, 'a> {
        builders::debug_map_new(self)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Display::fmt("an error occurred when formatting an argument", f)
    }
}

// Implementations of the core formatting traits

macro_rules! fmt_refs {
    ($($tr:ident),*) => {
        $(
        #[stable(feature = "rust1", since = "1.0.0")]
        impl<'a, T: ?Sized + $tr> $tr for &'a T {
            fn fmt(&self, f: &mut Formatter) -> Result { $tr::fmt(&**self, f) }
        }
        #[stable(feature = "rust1", since = "1.0.0")]
        impl<'a, T: ?Sized + $tr> $tr for &'a mut T {
            fn fmt(&self, f: &mut Formatter) -> Result { $tr::fmt(&**self, f) }
        }
        )*
    }
}

fmt_refs! { Debug, Display, Octal, Binary, LowerHex, UpperHex, LowerExp, UpperExp }

#[stable(feature = "rust1", since = "1.0.0")]
impl Debug for bool {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Display::fmt(self, f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Display for bool {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Display::fmt(if *self { "true" } else { "false" }, f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Debug for str {
    fn fmt(&self, f: &mut Formatter) -> Result {
        try!(write!(f, "\""));
        for c in self.chars().flat_map(|c| c.escape_default()) {
            try!(write!(f, "{}", c));
        }
        write!(f, "\"")
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Display for str {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.pad(self)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Debug for char {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use char::CharExt;
        try!(write!(f, "'"));
        for c in self.escape_default() {
            try!(write!(f, "{}", c));
        }
        write!(f, "'")
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Display for char {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut utf8 = [0; 4];
        let amt = self.encode_utf8(&mut utf8).unwrap_or(0);
        let s: &str = unsafe { mem::transmute(&utf8[..amt]) };
        Display::fmt(s, f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Pointer for *const T {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let old_width = f.width;
        let old_flags = f.flags;

        // The alternate flag is already treated by LowerHex as being special-
        // it denotes whether to prefix with 0x. We use it to work out whether
        // or not to zero extend, and then unconditionally set it to get the
        // prefix.
        if f.flags & 1 << (FlagV1::Alternate as u32) > 0 {
            f.flags |= 1 << (FlagV1::SignAwareZeroPad as u32);

            if let None = f.width {
                // The formats need two extra bytes, for the 0x
                if cfg!(target_pointer_width = "32") {
                    f.width = Some(10);
                } else {
                    f.width = Some(18);
                }
            }
        }
        f.flags |= 1 << (FlagV1::Alternate as u32);

        let ret = LowerHex::fmt(&(*self as usize), f);

        f.width = old_width;
        f.flags = old_flags;

        ret
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Pointer for *mut T {
    fn fmt(&self, f: &mut Formatter) -> Result {
        // FIXME(#23542) Replace with type ascription.
        #![allow(trivial_casts)]
        Pointer::fmt(&(*self as *const T), f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Pointer for &'a T {
    fn fmt(&self, f: &mut Formatter) -> Result {
        // FIXME(#23542) Replace with type ascription.
        #![allow(trivial_casts)]
        Pointer::fmt(&(*self as *const T), f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a, T> Pointer for &'a mut T {
    fn fmt(&self, f: &mut Formatter) -> Result {
        // FIXME(#23542) Replace with type ascription.
        #![allow(trivial_casts)]
        Pointer::fmt(&(&**self as *const T), f)
    }
}

// Common code of floating point Debug and Display.
fn float_to_str_common<T: float::MyFloat, F>(num: &T, precision: Option<usize>,
                                             post: F) -> Result
        where F : FnOnce(&str) -> Result {
    let digits = match precision {
        Some(i) => float::DigExact(i),
        None => float::DigMax(6),
    };
    float::float_to_str_bytes_common(num.abs(),
                                     digits,
                                     float::ExpNone,
                                     false,
                                     post)
}

macro_rules! floating { ($ty:ident) => {

    #[stable(feature = "rust1", since = "1.0.0")]
    impl Debug for $ty {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            float_to_str_common(self, fmt.precision, |absolute| {
                // is_positive() counts -0.0 as negative
                fmt.pad_integral(self.is_nan() || self.is_positive(), "", absolute)
            })
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl Display for $ty {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            float_to_str_common(self, fmt.precision, |absolute| {
                // simple comparison counts -0.0 as positive
                fmt.pad_integral(self.is_nan() || *self >= 0.0, "", absolute)
            })
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl LowerExp for $ty {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            let digits = match fmt.precision {
                Some(i) => float::DigExact(i),
                None => float::DigMax(6),
            };
            float::float_to_str_bytes_common(self.abs(),
                                             digits,
                                             float::ExpDec,
                                             false,
                                             |bytes| {
                fmt.pad_integral(self.is_nan() || *self >= 0.0, "", bytes)
            })
        }
    }

    #[stable(feature = "rust1", since = "1.0.0")]
    impl UpperExp for $ty {
        fn fmt(&self, fmt: &mut Formatter) -> Result {
            let digits = match fmt.precision {
                Some(i) => float::DigExact(i),
                None => float::DigMax(6),
            };
            float::float_to_str_bytes_common(self.abs(),
                                             digits,
                                             float::ExpDec,
                                             true,
                                             |bytes| {
                fmt.pad_integral(self.is_nan() || *self >= 0.0, "", bytes)
            })
        }
    }
} }
floating! { f32 }
floating! { f64 }

// Implementation of Display/Debug for various core types

#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Debug for *const T {
    fn fmt(&self, f: &mut Formatter) -> Result { Pointer::fmt(self, f) }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl<T> Debug for *mut T {
    fn fmt(&self, f: &mut Formatter) -> Result { Pointer::fmt(self, f) }
}

macro_rules! peel {
    ($name:ident, $($other:ident,)*) => (tuple! { $($other,)* })
}

macro_rules! tuple {
    () => ();
    ( $($name:ident,)+ ) => (
        #[stable(feature = "rust1", since = "1.0.0")]
        impl<$($name:Debug),*> Debug for ($($name,)*) {
            #[allow(non_snake_case, unused_assignments)]
            fn fmt(&self, f: &mut Formatter) -> Result {
                try!(write!(f, "("));
                let ($(ref $name,)*) = *self;
                let mut n = 0;
                $(
                    if n > 0 {
                        try!(write!(f, ", "));
                    }
                    try!(write!(f, "{:?}", *$name));
                    n += 1;
                )*
                if n == 1 {
                    try!(write!(f, ","));
                }
                write!(f, ")")
            }
        }
        peel! { $($name,)* }
    )
}

tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: Debug> Debug for [T] {
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.iter().fold(f.debug_list(), |b, e| b.entry(e)).finish()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl Debug for () {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.pad("()")
    }
}
impl<T> Debug for PhantomData<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.pad("PhantomData")
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: Copy + Debug> Debug for Cell<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Cell {{ value: {:?} }}", self.get())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<T: ?Sized + Debug> Debug for RefCell<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self.borrow_state() {
            BorrowState::Unused | BorrowState::Reading => {
                write!(f, "RefCell {{ value: {:?} }}", self.borrow())
            }
            BorrowState::Writing => write!(f, "RefCell {{ <borrowed> }}"),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'b, T: ?Sized + Debug> Debug for Ref<'b, T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Debug::fmt(&**self, f)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'b, T: ?Sized + Debug> Debug for RefMut<'b, T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        Debug::fmt(&*(self.deref()), f)
    }
}

// If you expected tests to be here, look instead at the run-pass/ifmt.rs test,
// it's a lot easier than creating all of the rt::Piece structures here.
