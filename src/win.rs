use crate::{MacInfo, ResultType};
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::shared::ws2def::{AF_UNSPEC, SOCKADDR_IN};
use winapi::shared::inaddr::IN_ADDR;
use winapi::um::iptypes::{IP_ADAPTER_ADDRESSES_LH, GAA_FLAG_INCLUDE_PREFIX};
use winapi::um::iphlpapi::{GetAdaptersAddresses, GetBestInterface};
use std::mem;
use anyhow::bail;

pub fn get_mac() -> ResultType<MacInfo> {
    unsafe {
        // Use 8.8.8.8 (Google DNS) to get the default network adapter
        let sock_addr = SOCKADDR_IN {
            sin_family: AF_UNSPEC as u16,
            sin_port: 0,
            sin_addr: IN_ADDR { S_un: mem::transmute(0x08080808u32) }, // 8.8.8.8
            sin_zero: [0; 8],
        };
        let mut if_index: u32 = 0;
        
        // Get the best interface index
        if GetBestInterface(mem::transmute(sock_addr.sin_addr), &mut if_index) == ERROR_SUCCESS {
            let mut buffer_length: u32 = 0;
            let _ = GetAdaptersAddresses(
                AF_UNSPEC as u32,
                GAA_FLAG_INCLUDE_PREFIX,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut buffer_length,
            );

            let mut buffer = vec![0u8; buffer_length as usize];
            let addresses = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

            if GetAdaptersAddresses(
                AF_UNSPEC as u32,
                GAA_FLAG_INCLUDE_PREFIX,
                std::ptr::null_mut(),
                addresses,
                &mut buffer_length,
            ) == ERROR_SUCCESS {
                let mut current_addresses = addresses;

                while !current_addresses.is_null() {
                    let adapter = &*current_addresses;
                    
                    // Match the found interface index
                    if adapter.u.s().IfIndex == if_index {
                        let name = String::from_utf16_lossy(
                            std::slice::from_raw_parts(
                                adapter.Description,
                                wcslen(adapter.Description)
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