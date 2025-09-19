use fire_config::TailscaleOptions;
use firecracker_vm::{mac::generate_unique_mac, types::VmOptions};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct MicroVM {
    pub id: String,
    pub name: String,
    pub image: String,
    pub vcpu: u8,
    pub memory: u32,
    pub vmlinux: String,
    pub rootfs: String,
    pub boot_args: String,
    pub status: String,
    pub ssh_keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct StartMicroVM {
    pub tailscale_auth_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateMicroVM {
    pub name: Option<String>,
    pub vcpus: Option<u8>,
    pub memory: Option<u16>,
    pub image: Option<String>,
    pub vmlinux: Option<String>,
    pub rootfs: Option<String>,
    pub boot_args: Option<String>,
    pub ssh_keys: Option<Vec<String>>,
    pub start: Option<bool>,
    pub tailscale_auth_key: Option<String>,
}

impl Into<VmOptions> for CreateMicroVM {
    fn into(self) -> VmOptions {
        VmOptions {
            debian: self
                .image
                .as_ref()
                .map(|img| img == "debian")
                .unwrap_or(false)
                .then_some(true),
            alpine: self
                .image
                .as_ref()
                .map(|img| img == "alpine")
                .unwrap_or(false)
                .then_some(true),
            ubuntu: self
                .image
                .as_ref()
                .map(|img| img == "ubuntu")
                .unwrap_or(false)
                .then_some(true),
            nixos: self
                .image
                .as_ref()
                .map(|img| img == "nixos")
                .unwrap_or(false)
                .then_some(true),
            fedora: self
                .image
                .as_ref()
                .map(|img| img == "fedora")
                .unwrap_or(false)
                .then_some(true),
            gentoo: self
                .image
                .as_ref()
                .map(|img| img == "gentoo")
                .unwrap_or(false)
                .then_some(true),
            slackware: self
                .image
                .as_ref()
                .map(|img| img == "slackware")
                .unwrap_or(false)
                .then_some(true),
            opensuse: self
                .image
                .as_ref()
                .map(|img| img == "opensuse")
                .unwrap_or(false)
                .then_some(true),
            opensuse_tumbleweed: self
                .image
                .as_ref()
                .map(|img| img == "opensuse-tumbleweed")
                .unwrap_or(false)
                .then_some(true),
            almalinux: self
                .image
                .as_ref()
                .map(|img| img == "almalinux")
                .unwrap_or(false)
                .then_some(true),
            rockylinux: self
                .image
                .as_ref()
                .map(|img| img == "rockylinux")
                .unwrap_or(false)
                .then_some(true),
            archlinux: self
                .image
                .as_ref()
                .map(|img| img == "archlinux")
                .unwrap_or(false)
                .then_some(true),
            vcpu: self.vcpus.unwrap_or(1) as u16,
            memory: self.memory.unwrap_or(512),
            vmlinux: self.vmlinux,
            rootfs: self.rootfs,
            bootargs: self.boot_args,
            mac_address: generate_unique_mac(),
            tailscale: self.tailscale_auth_key.map(|key| TailscaleOptions {
                auth_key: Some(key),
            }),
            ..Default::default()
        }
    }
}
