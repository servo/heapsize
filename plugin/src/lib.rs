/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Attributes this crate provides:
//!
//!  - `#[derive(HeapSizeOf)]` : Auto-derives an implementation of `HeapSizeOf` for a struct or
//!    enum.

#![feature(plugin_registrar, quote, plugin, box_syntax, rustc_private)]

#[macro_use]
extern crate syntax;
#[macro_use]
extern crate rustc_plugin;

use rustc_plugin::Registry;
use syntax::ext::base::*;

use syntax::parse::token::intern;

// Public for documentation to show up
/// Handles the auto-deriving for `#[derive(HeapSizeOf)]`
pub mod heap_size;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("derive_HeapSizeOf"), MultiDecorator(box heap_size::expand_heap_size));
}
