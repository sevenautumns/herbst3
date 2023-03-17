use std::str::FromStr;

use anyhow::Result;
use log::{debug, trace};

use crate::cli::ShiftDirection;
use crate::helper::Geometry;
use crate::parser::LayoutType;

pub fn get_focused_frame_index() -> Result<Vec<u8>> {
    let mut index_cmd = std::process::Command::new("herbstclient");
    let index_cmd = index_cmd
        .arg("get_attr")
        .arg("clients.focus.parent_frame.index");
    debug!("Execute get_attr: {index_cmd:?}");
    let output = index_cmd
        .output()?
        .stdout
        .iter()
        .filter_map(|i| match i {
            48 => Some(0u8),
            49 => Some(1u8),
            _ => None,
        })
        .collect::<Vec<_>>();
    trace!("Got output: {:?}", &output);
    Ok(output)
}

pub fn monitor_in_dir_exists(dir: ShiftDirection) -> bool {
    let dir = match dir {
        ShiftDirection::Right => "-r",
        ShiftDirection::Left => "-l",
        ShiftDirection::Up => "-u",
        ShiftDirection::Down => "-d",
    };

    let mut status_cmd = std::process::Command::new("herbstclient");
    let status_cmd = status_cmd.arg("monitor_rect").arg(dir);
    debug!("Execute monitor_rect: {status_cmd:?}");
    let status = status_cmd.status();
    status.is_ok()
}

pub fn get_layout() -> Result<String> {
    let mut layout_cmd = std::process::Command::new("herbstclient");
    let layout_cmd = layout_cmd.arg("dump");
    debug!("Execute dump: {layout_cmd:?}");
    let output = String::from_utf8(layout_cmd.output()?.stdout)?;
    trace!("Got output: {output}");
    Ok(output)
}

pub fn get_focused_frame_client_count() -> Result<usize> {
    let mut count_cmd = std::process::Command::new("herbstclient");
    let count_cmd = count_cmd.arg("get_attr").arg("tags.focus.curframe_wcount");
    debug!("Execute get_attr: {count_cmd:?}");
    let mut output = String::from_utf8(count_cmd.output()?.stdout)?;
    filter_control(&mut output);
    trace!("Got output: {output}");
    Ok(output.parse::<usize>()?)
}

pub fn get_focused_client_index() -> Result<usize> {
    let mut index_cmd = std::process::Command::new("herbstclient");
    let index_cmd = index_cmd.arg("get_attr").arg("tags.focus.curframe_windex");
    debug!("Execute get_attr: {index_cmd:?}");
    let mut output = String::from_utf8(index_cmd.output()?.stdout)?;
    filter_control(&mut output);
    trace!("Got output: {output}");
    Ok(output.parse::<usize>()?)
}

pub fn get_focused_frame_algorithm() -> Result<LayoutType> {
    let mut algo_cmd = std::process::Command::new("herbstclient");
    let algo_cmd = algo_cmd
        .arg("get_attr")
        .arg("clients.focus.parent_frame.algorithm");
    debug!("Execute get_attr: {algo_cmd:?}");
    let mut output = String::from_utf8(algo_cmd.output()?.stdout)?;
    filter_control(&mut output);
    trace!("Got output: {output}");
    Ok(LayoutType::from_str(&output)?)
}

pub fn create_split(frame_index: &[u8], dir: ShiftDirection, ratio: f32) -> Result<()> {
    let index = index_array_to_string(frame_index);
    let mut split_cmd = std::process::Command::new("herbstclient");
    let split_cmd = split_cmd
        .arg("split")
        .arg(dir.to_string())
        .arg(ratio.to_string())
        .arg(index);
    debug!("Execute split: {split_cmd:?}");
    split_cmd.output()?;
    Ok(())
}

pub fn get_focused_frame_geometry() -> Result<Geometry> {
    let mut geometry_cmd = std::process::Command::new("herbstclient");
    let geometry_cmd = geometry_cmd
        .arg("get_attr")
        .arg("clients.focus.parent_frame.content_geometry");
    debug!("Execute get_attr: {geometry_cmd:?}");
    let mut output = String::from_utf8(geometry_cmd.output()?.stdout)?;
    filter_control(&mut output);
    trace!("Got output: {output}");
    Geometry::from_str(&output)
}

pub fn get_focused_client_geometry() -> Result<Geometry> {
    let mut geometry_cmd = std::process::Command::new("herbstclient");
    let geometry_cmd = geometry_cmd
        .arg("get_attr")
        .arg("clients.focus.decoration_geometry");
    debug!("Execute get_attr: {geometry_cmd:?}");
    let mut output = String::from_utf8(geometry_cmd.output()?.stdout)?;
    filter_control(&mut output);
    trace!("Got output: {output}");
    Geometry::from_str(&output)
}

pub fn shift_focused_window(dir: ShiftDirection) -> Result<()> {
    let mut shift_cmd = std::process::Command::new("herbstclient");
    let shift_cmd = shift_cmd
        .arg("shift")
        .arg(dir.to_string())
        .arg("--level=all");
    debug!("Execute shift: {shift_cmd:?}");
    shift_cmd.output()?;
    Ok(())
}

pub fn shift_focused_window_remove_frame(dir: ShiftDirection) -> Result<()> {
    let mut shift_cmd = std::process::Command::new("herbstclient");
    let shift_cmd = shift_cmd
        .arg("chain")
        .arg("-")
        .arg("remove")
        .arg("-")
        .arg("shift")
        .arg(dir.to_string())
        .arg("--level=all");
    debug!("Execute shift: {shift_cmd:?}");
    shift_cmd.output()?;
    Ok(())
}

fn index_array_to_string(frame_index: &[u8]) -> String {
    frame_index
        .iter()
        .map(|i| i.to_string())
        .collect::<String>()
}

fn filter_control(str: &mut String) {
    *str = str
        .chars()
        .filter(|c| !c.is_ascii_control())
        .collect::<String>();
}
