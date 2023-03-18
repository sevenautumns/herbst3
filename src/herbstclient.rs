use std::process::Output;
use std::str::FromStr;

use anyhow::{bail, Context, Result};
use log::{debug, trace, warn};
use strum::Display;

use crate::cli::ShiftDirection;
use crate::helper::Geometry;
use crate::parser::LayoutType;

#[derive(Debug, Display, Clone, Copy, PartialEq)]
#[strum(serialize_all = "camelCase")]
pub enum SplitDirection {
    Right,
    Left,
    Top,
    Bottom,
}

impl From<ShiftDirection> for SplitDirection {
    fn from(dir: ShiftDirection) -> Self {
        match dir {
            ShiftDirection::Right => Self::Right,
            ShiftDirection::Left => Self::Left,
            ShiftDirection::Up => Self::Top,
            ShiftDirection::Down => Self::Bottom,
        }
    }
}

pub fn get_focused_frame_index() -> Result<Vec<u8>> {
    let mut index_cmd = std::process::Command::new("herbstclient");
    let index_cmd = index_cmd
        .arg("get_attr")
        .arg("clients.focus.parent_frame.index");
    debug!("Execute get_attr: {index_cmd:?}");
    let output = process_output(index_cmd.output()?)?;
    let index = output
        .chars()
        .map(|c| {
            c.to_digit(10)
                .map(|i| i as u8)
                .context("Unexpected number: {c}")
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(index)
}

pub fn monitor_in_dir_exists(dir: ShiftDirection) -> Result<bool> {
    let dir = match dir {
        ShiftDirection::Right => "-r",
        ShiftDirection::Left => "-l",
        ShiftDirection::Up => "-u",
        ShiftDirection::Down => "-d",
    };

    let mut status_cmd = std::process::Command::new("herbstclient");
    let status_cmd = status_cmd.arg("monitor_rect").arg(dir);
    debug!("Execute monitor_rect: {status_cmd:?}");
    let output = process_output(status_cmd.output()?);
    Ok(output.is_ok())
}

pub fn get_layout() -> Result<String> {
    let mut layout_cmd = std::process::Command::new("herbstclient");
    let layout_cmd = layout_cmd.arg("dump");
    debug!("Execute dump: {layout_cmd:?}");
    let output = process_output(layout_cmd.output()?)?;
    Ok(output)
}

pub fn get_focused_frame_client_count() -> Result<usize> {
    let mut count_cmd = std::process::Command::new("herbstclient");
    let count_cmd = count_cmd.arg("get_attr").arg("tags.focus.curframe_wcount");
    debug!("Execute get_attr: {count_cmd:?}");
    let output = process_output(count_cmd.output()?)?;
    Ok(output.parse::<usize>()?)
}

pub fn get_focused_client_index() -> Result<usize> {
    let mut index_cmd = std::process::Command::new("herbstclient");
    let index_cmd = index_cmd.arg("get_attr").arg("tags.focus.curframe_windex");
    debug!("Execute get_attr: {index_cmd:?}");
    let output = process_output(index_cmd.output()?)?;
    Ok(output.parse::<usize>()?)
}

pub fn get_focused_frame_algorithm() -> Result<LayoutType> {
    let mut algo_cmd = std::process::Command::new("herbstclient");
    let algo_cmd = algo_cmd
        .arg("get_attr")
        .arg("clients.focus.parent_frame.algorithm");
    debug!("Execute get_attr: {algo_cmd:?}");
    let output = process_output(algo_cmd.output()?)?;
    Ok(LayoutType::from_str(&output)?)
}

pub fn create_split(frame_index: &[u8], dir: ShiftDirection, ratio: f32) -> Result<()> {
    let index = index_array_to_string(frame_index);
    let mut split_cmd = std::process::Command::new("herbstclient");
    let split_cmd = split_cmd
        .arg("split")
        .arg(SplitDirection::from(dir).to_string())
        .arg(ratio.to_string())
        .arg(index);
    debug!("Execute split: {split_cmd:?}");
    process_output(split_cmd.output()?)?;

    Ok(())
}

pub fn get_focused_frame_geometry() -> Result<Geometry> {
    let mut geometry_cmd = std::process::Command::new("herbstclient");
    let geometry_cmd = geometry_cmd
        .arg("get_attr")
        .arg("clients.focus.parent_frame.content_geometry");
    debug!("Execute get_attr: {geometry_cmd:?}");
    let output = process_output(geometry_cmd.output()?)?;
    Geometry::from_str(&output)
}

pub fn get_focused_client_geometry() -> Result<Geometry> {
    let mut geometry_cmd = std::process::Command::new("herbstclient");
    let geometry_cmd = geometry_cmd
        .arg("get_attr")
        .arg("clients.focus.decoration_geometry");
    debug!("Execute get_attr: {geometry_cmd:?}");
    let output = process_output(geometry_cmd.output()?)?;
    Geometry::from_str(&output)
}

pub fn shift_focused_window(dir: ShiftDirection, frame: bool) -> Result<()> {
    let mut shift_cmd = std::process::Command::new("herbstclient");
    let mut shift_cmd = shift_cmd.arg("shift").arg(dir.to_string());
    if frame {
        shift_cmd = shift_cmd.arg("--level=frame");
    } else {
        shift_cmd = shift_cmd.arg("--level=all");
    }
    debug!("Execute shift: {shift_cmd:?}");
    process_output(shift_cmd.output()?)?;
    Ok(())
}

pub fn shift_focused_window_remove_frame(dir: ShiftDirection, frame: bool) -> Result<()> {
    let mut shift_cmd = std::process::Command::new("herbstclient");
    let mut shift_cmd = shift_cmd
        .arg("chain")
        .arg("-")
        .arg("remove")
        .arg("-")
        .arg("shift")
        .arg(dir.to_string());
    if frame {
        shift_cmd = shift_cmd.arg("--level=frame");
    } else {
        shift_cmd = shift_cmd.arg("--level=all");
    }
    debug!("Execute shift: {shift_cmd:?}");
    process_output(shift_cmd.output()?)?;
    Ok(())
}

fn index_array_to_string(frame_index: &[u8]) -> String {
    frame_index
        .iter()
        .map(|i| i.to_string())
        .collect::<String>()
}

fn filter_control(str: &str) -> String {
    str.chars()
        .filter(|c| !c.is_ascii_control())
        .collect::<String>()
}

fn process_output(output: Output) -> Result<String> {
    let out = filter_control(&String::from_utf8(output.stdout)?);
    let err = filter_control(&String::from_utf8(output.stderr)?);
    if !out.is_empty() {
        trace!("stdout: {out}");
    }
    if !err.is_empty() {
        warn!("stderr: {err}")
    }
    if !output.status.success() {
        bail!("Error: {:?}", output.status);
    }

    Ok(out)
}
