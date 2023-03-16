use anyhow::Result;

pub fn get_focused_index() -> Result<Vec<u8>> {
    Ok(std::process::Command::new("herbstclient")
        .arg("get_attr")
        .arg("clients.focus.parent_frame.index")
        .output()?
        .stdout
        .iter()
        .filter_map(|i| match i {
            48 => Some(0u8),
            49 => Some(1u8),
            _ => None,
        })
        .collect::<Vec<_>>())
}

pub fn get_layout() -> Result<String> {
    Ok(String::from_utf8(
        std::process::Command::new("herbstclient")
            .arg("dump")
            .output()?
            .stdout,
    )?)
}
