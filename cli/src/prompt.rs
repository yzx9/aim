// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use aimcal_core::DateTimeAnchor;
use cliclack::{input, intro, note, outro};

const TIME_NOTE: &str = "\
• Relative time: 10s, 10m, 2h, 3d
• Specific time: 14:30
• Date:          2025-01-15
• DateTime:      2025-01-15 14:30
• Keywords:      now, today, tomorrow, yesterday";

pub fn prompt_time() -> Result<DateTimeAnchor, Box<dyn std::error::Error>> {
    intro("Time Anchor Input")?;
    note("Supported formats:", TIME_NOTE)?;

    let input_str: String = input("Enter time anchor:")
        .placeholder("e.g., 2h, tomorrow, 14:30")
        .interact()?;

    match DateTimeAnchor::from_str(&input_str) {
        Ok(anchor) => {
            outro("Time anchor accepted")?;
            Ok(anchor)
        }
        Err(e) => {
            outro("Invalid time anchor format")?;
            Err(e.into())
        }
    }
}

pub fn prompt_time_opt() -> Result<Option<DateTimeAnchor>, Box<dyn std::error::Error>> {
    intro("Time Anchor Input")?;
    note("Supported formats:", TIME_NOTE)?;

    let input_str: String = input("Enter time anchor (or empty to skip):")
        .placeholder("e.g., 2h, tomorrow, 14:30")
        .interact()?;

    if input_str.trim().is_empty() {
        outro("No time anchor provided, skipping.")?;
        return Ok(None);
    }

    match DateTimeAnchor::from_str(&input_str) {
        Ok(anchor) => {
            outro("Time anchor accepted")?;
            Ok(Some(anchor))
        }
        Err(e) => {
            outro("Invalid time anchor format")?;
            Err(e.into())
        }
    }
}
