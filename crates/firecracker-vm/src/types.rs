use fire_config::FireConfig;
use firecracker_prepare::Distro;

use crate::constants::{BRIDGE_DEV, FC_MAC, FIRECRACKER_SOCKET, TAP_DEV};

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
    pub bridge: String,
    pub tap: String,
    pub api_socket: String,
    pub mac_address: String,
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
            bridge: vm.bridge.unwrap_or(BRIDGE_DEV.into()),
            tap: vm.tap.unwrap_or(TAP_DEV.into()),
            api_socket: vm.api_socket.unwrap_or(FIRECRACKER_SOCKET.into()),
            mac_address: vm.mac.unwrap_or(FC_MAC.into()),
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
