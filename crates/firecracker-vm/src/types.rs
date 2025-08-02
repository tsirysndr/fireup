use firecracker_prepare::Distro;

#[derive(Default, Clone)]
pub struct VmOptions {
    pub debian: Option<bool>,
    pub alpine: Option<bool>,
    pub ubuntu: Option<bool>,
    pub nixos: Option<bool>,
    pub vcpu: u16,
    pub memory: u16,
    pub vmlinux: Option<String>,
    pub rootfs: Option<String>,
    pub bootargs: Option<String>,
}

impl Into<Distro> for VmOptions {
    fn into(self) -> Distro {
        if self.debian.unwrap_or(false) {
            Distro::Debian
        } else if self.alpine.unwrap_or(false) {
            Distro::Alpine
        } else if self.nixos.unwrap_or(false) {
            Distro::NixOS
        } else if self.ubuntu.unwrap_or(true) {
            Distro::Ubuntu
        } else {
            panic!("No valid distribution option provided.");
        }
    }
}
