use anyhow::{bail, Result};
use log::debug;

use crate::cli::ShiftDirection;
use crate::helper::can_focus_within_frame;
use crate::herbstclient::Herbstclient;
use crate::parser::{get_layout_stack, LayoutType};

pub fn shift(dir: ShiftDirection, frame: bool) -> Result<()> {
    let mut hc = Herbstclient::new()?;

    let initial_clients = hc.get_focused_frame_client_count()?;

    if initial_clients == 0 {
        bail!("Focused frame is empty")
    }

    let client_index = hc.get_focused_client_index()?;

    // If we can shift within the current frame, do so
    if !frame && can_focus_within_frame(&mut hc, initial_clients, client_index, dir)? {
        debug!("Can be shifted within focused frame");
        hc.shift_focused_window(dir, false)?;
        return Ok(());
    }

    let source_index = hc.get_focused_frame_index()?;
    let layout_stack = get_layout_stack(&mut hc, &source_index)?;

    // Find index to split
    let split = find_split(initial_clients, dir, &source_index, &layout_stack);
    debug!("Split: {split:?}");

    match split {
        SplitAction::Split(index) => {
            // If we want to split, do so first
            hc.create_split(&index, dir, 0.5)?;
        }
        // Nothing is required if we just want to move locally
        SplitAction::MoveableLocal => {}
        SplitAction::MoveableGlobal => {
            // if no monitor in desired direction exists, bail
            if !hc.monitor_in_dir_exists(dir)? {
                bail!("No monitor in {dir} direction")
            }
        }
    }

    if initial_clients <= 1 {
        hc.shift_focused_window_remove_frame(dir, frame)?;
    } else {
        hc.shift_focused_window(dir, frame)?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum SplitAction {
    Split(Vec<u8>),
    MoveableLocal,
    MoveableGlobal,
}

pub fn find_split(
    clients: usize,
    dir: ShiftDirection,
    index: &[u8],
    layout_stack: &[LayoutType],
) -> SplitAction {
    let movable_index = match dir {
        ShiftDirection::Right | ShiftDirection::Down => 0,
        ShiftDirection::Left | ShiftDirection::Up => 1,
    };
    let target_layout_type = match dir {
        ShiftDirection::Right | ShiftDirection::Left => LayoutType::Horizontal,
        ShiftDirection::Up | ShiftDirection::Down => LayoutType::Vertical,
    };

    // In case the root-node itself is a "clients" container
    // and there is more than one client: Split it
    if clients > 1 {
        return SplitAction::Split(index.to_vec());
    }

    for (e, (i, l)) in index.iter().zip(layout_stack.iter()).enumerate().rev() {
        if movable_index.eq(i) && target_layout_type.eq(l) {
            debug!("No split, because {target_layout_type} layout is movable in {dir} direction");
            return SplitAction::MoveableLocal;
        }
        if target_layout_type.ne(l) {
            debug!("Found {l} layout for split");
            return SplitAction::Split(index[0..e].to_vec());
        }
    }

    SplitAction::MoveableGlobal
}
