// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use aimcal_cli::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    run().await
}
