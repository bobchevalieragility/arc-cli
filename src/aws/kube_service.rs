#[derive(Clone, Debug)]
pub struct KubeService {
    pub name: String,
    pub port: u16,
}

impl KubeService {
    pub fn new(name: String, port: u16) -> KubeService {
        KubeService { name, port }
    }
}
