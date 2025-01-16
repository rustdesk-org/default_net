use crate::{MacInfo, ResultType};
use anyhow::anyhow;
use std::fs;
use std::path::Path;

pub fn get_mac() -> ResultType<MacInfo> {
    // Read /proc/net/route to get the default network interface
    let route_content = fs::read_to_string("/proc/net/route")?;

    // Find the network interface with default route
    let default_interface = route_content
        .lines()
        .skip(1) // Skip header line
        .find(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            fields.len() > 1 && fields[1] == "00000000" // Destination address is 0.0.0.0 for default route
        })
        .and_then(|line| line.split_whitespace().next())
        .ok_or_else(|| anyhow!("Could not find default network interface"))?;

    // Build the path to MAC address file
    let mac_path = format!("/sys/class/net/{}/address", default_interface);

    if !Path::new(&mac_path).exists() {
        return Err(anyhow!("MAC address file does not exist: {}", mac_path));
    }

    // Read MAC address
    let mac = fs::read_to_string(&mac_path)?.trim().to_string();

    Ok(MacInfo {
        name: default_interface.to_string(),
        addr: mac,
    })
}
