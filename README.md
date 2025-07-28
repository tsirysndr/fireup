# Fireup

[![ci](https://github.com/tsirysndr/fireup/actions/workflows/ci.yml/badge.svg)](https://github.com/tsirysndr/fireup/actions/workflows/ci.yml)

`fireup` is a tool designed to simplify the process of setting up and managing Firecracker microVMs. It automates the preparation of the necessary files, including kernel images, root filesystems, and SSH keys, to quickly get you started with [Firecracker](https://firecracker-microvm.github.io/).

![Fireup Preview](./preview.png)

## Features

- **Quick Setup**: Prepares linux kernel, Ubuntu rootfs, and SSH keys in one command.
- **Seamless VM Management**: Start, stop, and monitor Firecracker microVMs with intuitive subcommands.
- **Network Configuration**: Automatically sets up TAP devices, IP forwarding, and NAT for connectivity.
- **SSH Access**: Easily connect to the microVM via SSH.
- **Cross-Architecture Support**: Supports x86_64 and aarch64 with automatic detection.
- **Robust Error Handling**: Clear error messages using anyhow for easy debugging.

## Subcommands
- `up`: Starts the Firecracker microVM, preparing assets and configuring the network if needed.
- `down`: Stops the running Firecracker microVM.
- `status`: Checks the status of the Firecracker microVM (running, stopped, or errored).
- `logs`: Displays the logs of the Firecracker microVM from the log file.
- `ssh`: Connects to the Firecracker microVM via SSH.
- `help`: Prints help information for the CLI or specific subcommands.
