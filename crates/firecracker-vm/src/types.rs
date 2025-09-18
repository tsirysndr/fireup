use fire_config::{EtcdConfig, FireConfig};
use firecracker_prepare::Distro;

use crate::constants::{BRIDGE_DEV, FC_MAC, FIRECRACKER_SOCKET};

#[derive(Default, Clone)]
pub struct VmOptions {
    pub debian: Option<bool>,
    pub alpine: Option<bool>,
    pub ubuntu: Option<bool>,
    pub nixos: Option<bool>,
    pub fedora: Option<bool>,
    pub gentoo: Option<bool>,
    pub slackware: Option<bool>,
    pub opensuse: Option<bool>,
    pub opensuse_tumbleweed: Option<bool>,
    pub almalinux: Option<bool>,
    pub rockylinux: Option<bool>,
    pub archlinux: Option<bool>,
    pub vcpu: u16,
    pub memory: u16,
    pub vmlinux: Option<String>,
    pub rootfs: Option<String>,
    pub bootargs: Option<String>,
    pub bridge: String,
    pub tap: String,
    pub api_socket: String,
    pub mac_address: String,
    pub etcd: Option<EtcdConfig>,
    pub ssh_keys: Option<Vec<String>>,
}

impl From<FireConfig> for VmOptions {
    fn from(config: FireConfig) -> Self {
        let vm = config.vm;
        VmOptions {
            debian: Some(config.distro == Distro::Debian),
            alpine: Some(config.distro == Distro::Alpine),
            ubuntu: Some(config.distro == Distro::Ubuntu),
            nixos: Some(config.distro == Distro::NixOS),
            fedora: Some(config.distro == Distro::Fedora),
            gentoo: Some(config.distro == Distro::Gentoo),
            slackware: Some(config.distro == Distro::Slackware),
            opensuse: Some(config.distro == Distro::Opensuse),
            opensuse_tumbleweed: Some(config.distro == Distro::OpensuseTumbleweed),
            almalinux: Some(config.distro == Distro::Almalinux),
            rockylinux: Some(config.distro == Distro::RockyLinux),
            archlinux: Some(config.distro == Distro::Archlinux),
            vcpu: vm.vcpu.unwrap_or(num_cpus::get() as u16),
            memory: vm.memory.unwrap_or(512),
            vmlinux: vm.vmlinux,
            rootfs: vm.rootfs,
            bootargs: vm.boot_args,
            bridge: vm.bridge.unwrap_or(BRIDGE_DEV.into()),
            tap: vm.tap.unwrap_or("".into()),
            api_socket: vm.api_socket.unwrap_or(FIRECRACKER_SOCKET.into()),
            mac_address: vm.mac.unwrap_or(FC_MAC.into()),
            etcd: config.etcd.clone(),
            ssh_keys: vm.ssh_keys.clone(),
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
        } else if self.fedora.unwrap_or(false) {
            Distro::Fedora
        } else if self.gentoo.unwrap_or(false) {
            Distro::Gentoo
        } else if self.slackware.unwrap_or(false) {
            Distro::Slackware
        } else if self.opensuse.unwrap_or(false) {
            Distro::Opensuse
        } else if self.opensuse_tumbleweed.unwrap_or(false) {
            Distro::OpensuseTumbleweed
        } else if self.almalinux.unwrap_or(false) {
            Distro::Almalinux
        } else if self.rockylinux.unwrap_or(false) {
            Distro::RockyLinux
        } else if self.archlinux.unwrap_or(false) {
            Distro::Archlinux
        } else if self.ubuntu.unwrap_or(true) {
            Distro::Ubuntu
        } else {
            panic!("No valid distribution option provided.");
        }
    }
}
