use anyhow::{bail, Result};
use log::debug;

use crate::cli::ShiftDirection;
use crate::helper::can_focus_within_frame;
use crate::herbstclient::{
    create_split, get_focused_client_index, get_focused_frame_client_count,
    get_focused_frame_index, monitor_in_dir_exists, shift_focused_window,
    shift_focused_window_remove_frame,
};
use crate::parser::{get_layout_stack, LayoutType};

pub fn shift(dir: ShiftDirection) -> Result<()> {
    let initial_clients = get_focused_frame_client_count()?;

    if initial_clients == 0 {
        bail!("Focused frame is empty")
    }

    let client_index = get_focused_client_index()?;

    // If we can shift within the current frame, do so
    if can_focus_within_frame(initial_clients, client_index, dir)? {
        debug!("Can be shifted within focused frame");
        shift_focused_window(dir)?;
        return Ok(());
    }

    let source_index = get_focused_frame_index()?;
    let layout_stack = get_layout_stack(&source_index)?;

    // Find index to split
    let split = find_split(dir, &source_index, &layout_stack);
    debug!("Split: {split:?}");

    match split {
        SplitAction::Split(index) => {
            // If we want to split, do so first
            create_split(&index, dir, 0.5)?;
        }
        // Nothing is required if we just want to move locally
        SplitAction::MoveableLocal => {}
        SplitAction::MoveableGlobal => {
            // if no monitor in desired direction exists, bail
            if monitor_in_dir_exists(dir) {
                bail!("No monitor in {dir} direction")
            }
        }
    }

    if initial_clients <= 1 {
        shift_focused_window_remove_frame(dir)?;
    } else {
        shift_focused_window(dir)?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum SplitAction {
    Split(Vec<u8>),
    MoveableLocal,
    MoveableGlobal,
}

pub fn find_split(dir: ShiftDirection, index: &[u8], layout_stack: &[LayoutType]) -> SplitAction {
    let movable_index = match dir {
        ShiftDirection::Right | ShiftDirection::Down => 0,
        ShiftDirection::Left | ShiftDirection::Up => 1,
    };
    let target_layout_type = match dir {
        ShiftDirection::Right | ShiftDirection::Left => LayoutType::Horizontal,
        ShiftDirection::Up | ShiftDirection::Down => LayoutType::Vertical,
    };

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
