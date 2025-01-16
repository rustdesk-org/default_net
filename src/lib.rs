#[cfg(target_os = "macos")]
mod macos;

pub type ResultType<F, E = anyhow::Error> = anyhow::Result<F, E>;

#[derive(Debug)]
pub struct MacInfo {
    pub name: String,
    pub addr: String,
}

mod test {
    use super::*;

    #[test]
    fn test_get_mac() {
        let mac = macos::get_mac();
        println!("res: {:?}", mac);
    }
}
