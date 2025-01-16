#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod os;
#[cfg(target_os = "windows")]
#[path = "win.rs"]
mod os;

pub use os::get_mac;

pub type ResultType<F, E = anyhow::Error> = anyhow::Result<F, E>;

#[derive(Debug)]
pub struct MacInfo {
    pub name: String,
    pub addr: String,
}

mod test {

    #[test]
    fn test_get_mac() {
        let mac = super::get_mac();
        println!("res: {:?}", mac);
    }
}
