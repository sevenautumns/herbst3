use crate::{
    cli::ShiftDirection,
    herbstclient::get_focused_index,
    parser::{get_layout_stack, LayoutType},
};
use anyhow::{Context, Result};
use log::debug;

pub fn shift(dir: ShiftDirection) -> Result<()> {
    let frame_index = get_focused_index()?;
    let layout_stack = get_layout_stack(&frame_index)?;
    let split = find_split(dir, &frame_index, &layout_stack);
    debug!("Split: {split:?}");
    todo!()
}

pub fn find_split(
    dir: ShiftDirection,
    index: &[u8],
    layout_stack: &[LayoutType],
) -> Result<Option<String>> {
    debug!("index: {index:?}");
    debug!("layout_stack: {layout_stack:?}");
    let target_index = match dir {
        ShiftDirection::Right | ShiftDirection::Down => 1,
        ShiftDirection::Left | ShiftDirection::Up => 0,
    };
    let target_layout_type = match dir {
        ShiftDirection::Right | ShiftDirection::Left => LayoutType::Horizontal,
        ShiftDirection::Up | ShiftDirection::Down => LayoutType::Vertical,
    };
    if target_index.ne(index.last().context("")?)
        && target_layout_type.eq(layout_stack.last().context("")?)
    {
        debug!("No split, because source frame is free in desired direction");
        return Ok(None);
    }

    // for (e, (i, l)) in index.iter().zip(layout_stack.iter()).enumerate().rev() {
    //     if target_index.eq(i) && target_layout_type.eq(l) {
    //         debug!("Found match: i: {i} l: {l:?}");
    //         return Some(index[0..=e].iter().map(|i| i.to_string()).collect());
    //     }
    // }
    // if layout_stack[0] != target_layout_type {
    //     return Some(index[0].to_string());
    // }
    Ok(None)
}
