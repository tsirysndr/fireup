use fire_config::FireConfig;
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

impl From<FireConfig> for VmOptions {
    fn from(config: FireConfig) -> Self {
        let vm = config.vm;
        VmOptions {
            debian: Some(config.distro == Distro::Debian),
            alpine: Some(config.distro == Distro::Alpine),
            ubuntu: Some(config.distro == Distro::Ubuntu),
            nixos: Some(config.distro == Distro::NixOS),
            vcpu: vm.vcpu.unwrap_or(num_cpus::get() as u16),
            memory: vm.memory.unwrap_or(512),
            vmlinux: vm.vmlinux,
            rootfs: vm.rootfs,
            bootargs: vm.boot_args,
        }
    }
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
