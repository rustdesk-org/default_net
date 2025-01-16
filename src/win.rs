use crate::{MacInfo, ResultType};
use windows::Win32::NetworkManagement::IpHelper::{GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH, GetBestInterface};
use windows::Win32::NetworkManagement::IpHelper::GAA_FLAG_INCLUDE_PREFIX;
use windows::Win32::Networking::WinSock::{AF_UNSPEC, SOCKADDR_IN, IN_ADDR};
use anyhow::bail;

pub fn get_mac() -> ResultType<MacInfo> {
    unsafe {
        // Use 8.8.8.8 (Google DNS) to get the default network adapter
        let sock_addr = SOCKADDR_IN {
            sin_family: AF_UNSPEC,
            sin_port: 0,
            sin_addr: IN_ADDR { S_un: std::mem::transmute(0x08080808u32) }, // 8.8.8.8
            sin_zero: [0; 8],
        };
        let mut if_index: u32 = 0;
        
        // Get the best interface index
        if GetBestInterface(std::mem::transmute(sock_addr.sin_addr), &mut if_index) == 0 {
            let mut buffer_length: u32 = 0;
            let _ = GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                GAA_FLAG_INCLUDE_PREFIX,
                None,
                None,
                &mut buffer_length,
            );

            let mut buffer = vec![0u8; buffer_length as usize];
            let addresses = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

            if GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                GAA_FLAG_INCLUDE_PREFIX,
                None,
                Some(addresses),
                &mut buffer_length,
            ) == 0 {
                let mut current_addresses = addresses;

                while !current_addresses.is_null() {
                    let adapter = &*current_addresses;
                    
                    // Match the found interface index
                    if adapter.Anonymous1.Anonymous.IfIndex == if_index {
                        let name = String::from_utf16_lossy(
                            std::slice::from_raw_parts(
                                adapter.Description.0,
                                wcslen(adapter.Description.0)
                            )
                        );

                        let mac = &adapter.PhysicalAddress[..adapter.PhysicalAddressLength as usize];
                        let mac_string = format!(
                            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                        );

                        return Ok(MacInfo {
                            name,
                            addr: mac_string,
                        });
                    }
                    current_addresses = adapter.Next;
                }
            }
        }
    }
    bail!("no adapter found")
}

fn wcslen(ptr: *const u16) -> usize {
    let mut len = 0;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
    }
    len
}