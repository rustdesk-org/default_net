use crate::{MacInfo, ResultType};
use anyhow::bail;
use std::mem;
use winapi::shared::inaddr::IN_ADDR;
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::shared::ws2def::{AF_UNSPEC, SOCKADDR_IN};
use winapi::um::iphlpapi::{GetAdaptersAddresses, GetBestInterface};
use winapi::um::iptypes::{GAA_FLAG_INCLUDE_PREFIX, IP_ADAPTER_ADDRESSES_LH};

pub fn get_mac() -> ResultType<MacInfo> {
    unsafe {
        // Use 8.8.8.8 (Google DNS) to get the default network adapter
        let sock_addr = SOCKADDR_IN {
            sin_family: AF_UNSPEC as u16,
            sin_port: 0,
            sin_addr: IN_ADDR {
                S_un: mem::transmute(0x08080808u32),
            }, // 8.8.8.8
            sin_zero: [0; 8],
        };
        let mut if_index: u32 = 0;

        // Get the best interface index
        if GetBestInterface(mem::transmute(sock_addr.sin_addr), &mut if_index) == ERROR_SUCCESS {
            let mut buffer_length: u32 = 0;

            // First call to get buffer size
            let _result = GetAdaptersAddresses(
                AF_UNSPEC as u32,
                GAA_FLAG_INCLUDE_PREFIX,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut buffer_length,
            );

            // Check if buffer_length is reasonable (prevent potential OOM)
            if buffer_length == 0 || buffer_length > 1024 * 1024 {
                bail!("Invalid buffer length received");
            }

            let mut buffer = vec![0u8; buffer_length as usize];
            let addresses = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

            if GetAdaptersAddresses(
                AF_UNSPEC as u32,
                GAA_FLAG_INCLUDE_PREFIX,
                std::ptr::null_mut(),
                addresses,
                &mut buffer_length,
            ) == ERROR_SUCCESS
            {
                let mut current_addresses = addresses;

                while !current_addresses.is_null() {
                    let adapter = &*current_addresses;

                    if adapter.u.s().IfIndex == if_index {
                        // Check if Description pointer is valid
                        if adapter.Description.is_null() {
                            bail!("Adapter description is null");
                        }

                        // Limit the maximum string length to prevent potential buffer overrun
                        let desc_len = wcslen(adapter.Description).min(1024);

                        let name = String::from_utf16_lossy(std::slice::from_raw_parts(
                            adapter.Description,
                            desc_len,
                        ));

                        // Validate PhysicalAddressLength
                        if adapter.PhysicalAddressLength != 6 {
                            bail!("Invalid MAC address length");
                        }

                        let mac = &adapter.PhysicalAddress[..6];
                        let mac_string = format!(
                            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                        );

                        return Ok(MacInfo {
                            name,
                            addr: mac_string,
                        });
                    }
                    // Validate Next pointer before continuing
                    if current_addresses == adapter.Next {
                        bail!("Circular linked list detected");
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
        // Add maximum length check to prevent potential overflow
        while len < 2048 && !ptr.add(len).is_null() && *ptr.add(len) != 0 {
            len += 1;
        }
    }
    len
}
