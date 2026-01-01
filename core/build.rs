// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

fn main() {
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=src/localdb/migrations");
}
