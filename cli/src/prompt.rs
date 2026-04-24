// SPDX-FileCopyrightText: 2025-2026 Zexin Yuan <aim@yzx9.xyz>
//
// SPDX-License-Identifier: Apache-2.0

use std::io::IsTerminal;
use std::str::FromStr;

use aimcal_core::DateTimeAnchor;
use cliclack::{input, intro, note, outro, select};

const TIME_NOTE: &str = "\
• Relative time: 10s, 10m, 2h, 3d
• Specific time: 14:30
• Date:          2025-01-15
• DateTime:      2025-01-15 14:30
• Keywords:      now, today, tomorrow, yesterday";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevModeChoice {
    Exit,
    Normal,
    Dev,
}

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

pub fn prompt_dev_mode_choice() -> Result<DevModeChoice, Box<dyn std::error::Error>> {
    intro("AIM_DEV Detected")?;
    note(
        "Environment selection:",
        "Release build detected AIM_DEV in the environment.\nNormal environment ignores AIM_DEV and AIM_CONFIG for this run.",
    )?;

    let choice = select("Choose how to continue:")
        .item(DevModeChoice::Exit, "Exit", "Abort this run")
        .item(
            DevModeChoice::Normal,
            "Normal environment",
            "Use the standard config discovery flow",
        )
        .item(
            DevModeChoice::Dev,
            "DEV mode",
            "Keep using the development environment variables",
        )
        .initial_value(DevModeChoice::Normal)
        .interact()?;

    match choice {
        DevModeChoice::Exit => outro("Canceled")?,
        DevModeChoice::Normal => outro("Using normal environment")?,
        DevModeChoice::Dev => outro("Using DEV mode")?,
    }

    Ok(choice)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateChoice {
    UpdateExisting,
    CreateNew,
}

/// Check if stdout is a terminal (interactive mode).
pub fn is_terminal() -> bool {
    std::io::stdout().is_terminal()
}

pub fn prompt_duplicate_choice(
    item_kind: &str,
    existing_id: &str,
    summary: &str,
) -> Result<DuplicateChoice, Box<dyn std::error::Error>> {
    intro("Similar item found")?;
    note(
        "An item with the same summary already exists:",
        format!("{item_kind} #{existing_id}: {summary}"),
    )?;

    let choice = select("What would you like to do?")
        .item(
            DuplicateChoice::UpdateExisting,
            "Update existing",
            "Apply the new fields to the existing item",
        )
        .item(
            DuplicateChoice::CreateNew,
            "Create new",
            "Create a separate item anyway",
        )
        .initial_value(DuplicateChoice::UpdateExisting)
        .interact()?;

    match choice {
        DuplicateChoice::UpdateExisting => outro("Updating existing item")?,
        DuplicateChoice::CreateNew => outro("Creating new item")?,
    }

    Ok(choice)
}
