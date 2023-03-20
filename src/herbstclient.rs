use std::{
    ffi::CString, mem::MaybeUninit, os::unix::process::ExitStatusExt, process::ExitStatus,
    str::FromStr,
};

use anyhow::{bail, Context, Result};
use log::{debug, trace, warn};
use strum::Display;

use crate::{cli::ShiftDirection, helper::Geometry, parser::LayoutType};

include!(concat!(env!("OUT_DIR"), "/herbst_ipc.rs"));

#[allow(dead_code)]
struct CommandReturn {
    out: String,
    err: String,
    status: ExitStatus,
}

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

pub struct Herbstclient(*mut HCConnection);

impl Herbstclient {
    pub fn new() -> Result<Self> {
        let con = unsafe { hc_connect() };
        if unsafe { !hc_check_running(con) } {
            bail!("No running HerbstluftWM instance found")
        }
        Ok(Self(con))
    }

    pub fn get_focused_frame_index(&mut self) -> Result<Vec<u8>> {
        let args = [
            CString::new("get_attr")?,
            CString::new("clients.focus.parent_frame.index")?,
        ];
        let index = self
            .send_command(&args)?
            .out
            .chars()
            .map(|c| {
                c.to_digit(10)
                    .map(|i| i as u8)
                    .context("Unexpected number: {c}")
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(index)
    }

    pub fn monitor_in_dir_exists(&mut self, dir: ShiftDirection) -> Result<bool> {
        let dir = match dir {
            ShiftDirection::Right => "-r",
            ShiftDirection::Left => "-l",
            ShiftDirection::Up => "-u",
            ShiftDirection::Down => "-d",
        };

        let args = [CString::new("monitor_rect")?, CString::new(dir)?];
        Ok(self.send_command(&args).is_ok())
    }

    pub fn get_layout(&mut self) -> Result<String> {
        let args = [CString::new("dump")?];
        Ok(self.send_command(&args)?.out)
    }

    pub fn get_focused_frame_client_count(&mut self) -> Result<usize> {
        let args = [
            CString::new("get_attr")?,
            CString::new("tags.focus.curframe_wcount")?,
        ];
        Ok(self.send_command(&args)?.out.parse::<usize>()?)
    }

    pub fn get_focused_client_index(&mut self) -> Result<usize> {
        let args = [
            CString::new("get_attr")?,
            CString::new("tags.focus.curframe_windex")?,
        ];
        Ok(self.send_command(&args)?.out.parse::<usize>()?)
    }

    pub fn get_focused_frame_algorithm(&mut self) -> Result<LayoutType> {
        let args = [
            CString::new("get_attr")?,
            CString::new("clients.focus.parent_frame.algorithm")?,
        ];
        Ok(LayoutType::from_str(&self.send_command(&args)?.out)?)
    }

    pub fn create_split(
        &mut self,
        frame_index: &[u8],
        dir: ShiftDirection,
        ratio: f32,
    ) -> Result<()> {
        let args = [
            CString::new("split")?,
            CString::new(SplitDirection::from(dir).to_string())?,
            CString::new(ratio.to_string())?,
            CString::new(index_array_to_string(frame_index))?,
        ];
        self.send_command(&args)?;
        Ok(())
    }

    pub fn get_focused_frame_geometry(&mut self) -> Result<Geometry> {
        let args = [
            CString::new("get_attr")?,
            CString::new("clients.focus.parent_frame.content_geometry")?,
        ];
        Geometry::from_str(&self.send_command(&args)?.out)
    }

    pub fn get_focused_client_geometry(&mut self) -> Result<Geometry> {
        let args = [
            CString::new("get_attr")?,
            CString::new("clients.focus.decoration_geometry")?,
        ];
        Geometry::from_str(&self.send_command(&args)?.out)
    }

    pub fn shift_focused_window(&mut self, dir: ShiftDirection, frame: bool) -> Result<()> {
        let mut args = [
            CString::new("shift")?,
            CString::new(dir.to_string())?,
            CString::default(),
        ];
        if frame {
            args[2] = CString::new("--level=frame")?;
        } else {
            args[2] = CString::new("--level=all")?;
        }
        self.send_command(&args)?;
        Ok(())
    }

    pub fn shift_focused_window_remove_frame(
        &mut self,
        dir: ShiftDirection,
        frame: bool,
    ) -> Result<()> {
        let mut args = [
            CString::new("chain")?,
            CString::new("-")?,
            CString::new("remove")?,
            CString::new("-")?,
            CString::new("shift")?,
            CString::new(dir.to_string())?,
            CString::default(),
        ];
        if frame {
            args[6] = CString::new("--level=frame")?;
        } else {
            args[6] = CString::new("--level=all")?;
        }
        self.send_command(&args)?;
        Ok(())
    }

    fn send_command(&mut self, args: &[CString]) -> Result<CommandReturn> {
        debug!("Execute command: {args:?}");
        let mut out = MaybeUninit::uninit();
        let mut err = MaybeUninit::uninit();
        let mut status = MaybeUninit::uninit();
        let args = args.iter().map(|a| a.as_ptr()).collect::<Vec<_>>();
        unsafe {
            let success = hc_send_command(
                self.0,
                args.len() as i32,
                args.as_slice().as_ptr() as *mut _,
                out.as_mut_ptr(),
                err.as_mut_ptr(),
                status.as_mut_ptr(),
            );
            if !success {
                bail!("Send command failed")
            }
            let out = CString::from_raw(out.assume_init()).into_string()?;
            let err = CString::from_raw(err.assume_init()).into_string()?;
            let status = ExitStatus::from_raw(status.assume_init());
            if !out.is_empty() {
                trace!("stdout: {out}");
            }
            if !err.is_empty() {
                warn!("stderr: {err}")
            }
            if !status.success() {
                bail!("Error: {status:?}");
            }
            Ok(CommandReturn { out, err, status })
        }
    }
}

impl Drop for Herbstclient {
    fn drop(&mut self) {
        unsafe { hc_disconnect(self.0) }
    }
}

fn index_array_to_string(frame_index: &[u8]) -> String {
    frame_index
        .iter()
        .map(|i| i.to_string())
        .collect::<String>()
}
