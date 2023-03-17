use std::str::FromStr;

use anyhow::{Context, Result};

use crate::cli::ShiftDirection;
use crate::herbstclient::{
    get_focused_client_geometry, get_focused_frame_algorithm, get_focused_frame_geometry,
};
use crate::parser::LayoutType;

#[derive(Clone, Debug)]
pub struct Geometry {
    width: usize,
    height: usize,
    x: usize,
    y: usize,
}

impl FromStr for Geometry {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s
            .split(&['x', '+'][..])
            .map(|n| n.parse::<usize>())
            .collect::<Result<Vec<_>, _>>()?;
        let error = "Malformed geometry";
        Ok(Geometry {
            width: *s.first().context(error)?,
            height: *s.get(1).context(error)?,
            x: *s.get(2).context(error)?,
            y: *s.get(3).context(error)?,
        })
    }
}

impl Geometry {
    pub fn right_edge(&self) -> usize {
        self.width + self.x
    }

    pub fn left_edge(&self) -> usize {
        self.x
    }

    pub fn top_edge(&self) -> usize {
        self.y
    }

    pub fn bottom_edge(&self) -> usize {
        self.height + self.y
    }

    pub fn child_can_move(&self, child: &Geometry, dir: ShiftDirection) -> bool {
        match dir {
            ShiftDirection::Right => self.right_edge() > child.right_edge(),
            ShiftDirection::Left => self.left_edge() < child.left_edge(),
            ShiftDirection::Up => self.top_edge() < child.top_edge(),
            ShiftDirection::Down => self.bottom_edge() > child.bottom_edge(),
        }
    }
}

pub fn can_focus_within_frame(clients: usize, index: usize, dir: ShiftDirection) -> Result<bool> {
    let algo = get_focused_frame_algorithm()?;
    if let LayoutType::Max = algo {
        match dir {
            ShiftDirection::Right => return Ok(index < (clients - 1)),
            ShiftDirection::Left => return Ok(index > 0),
            ShiftDirection::Up | ShiftDirection::Down => return Ok(false),
        }
    }

    let frame = get_focused_frame_geometry()?;
    let client = get_focused_client_geometry()?;
    Ok(frame.child_can_move(&client, dir))
}
