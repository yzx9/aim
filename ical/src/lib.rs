// SPDX-FileCopyrightText: 2025 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

mod lexer;
mod parser;
mod property_value;

pub use parser::{Component, parse};
