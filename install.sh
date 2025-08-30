#!/usr/bin/env bash

set -e -o pipefail

readonly MAGENTA="$(tput setaf 5 2>/dev/null || echo '')"
readonly GREEN="$(tput setaf 2 2>/dev/null || echo '')"
readonly CYAN="$(tput setaf 6 2>/dev/null || echo '')"
readonly ORANGE="$(tput setaf 3 2>/dev/null || echo '')"
readonly NO_COLOR="$(tput sgr0 2>/dev/null || echo '')"

if ! command -v curl >/dev/null 2>&1; then
    echo "Error: curl is required to install fireup."
    exit 1
fi

if ! command -v tar >/dev/null 2>&1; then
    echo "Error: tar is required to install fireup."
    exit 1
fi

export PATH="$HOME/.local/bin:$PATH"

RELEASE_URL="https://api.github.com/repos/firecracker-microvm/firecracker/releases/latest"

function detect_os() {
  # Determine the operating system
  OS=$(uname -s)
  if [ "$OS" = "Linux" ]; then
      # Determine the CPU architecture
      ARCH=$(uname -m)
      if [ "$ARCH" = "aarch64" ]; then
          ASSET_NAME="-aarch64.tgz"
      elif [ "$ARCH" = "x86_64" ]; then
          ASSET_NAME="-x86_64.tgz"
      else
          echo "Unsupported architecture: $ARCH"
          exit 1
      fi
  else
      echo "Unsupported operating system: $OS"
      echo "This script only supports Linux."
      exit 1
  fi;
}

detect_os

DOWNLOAD_URL=$(curl -sSL "$RELEASE_URL" | grep -o "browser_download_url.*firecracker-.*$ASSET_NAME\"" | cut -d ' ' -f 2)

DOWNLOAD_URL=`echo $DOWNLOAD_URL | tr -d '\"'`

ASSET_NAME=$(basename $DOWNLOAD_URL)

curl -SL $DOWNLOAD_URL -o /tmp/$ASSET_NAME

tar -xzf /tmp/$ASSET_NAME -C /tmp

mkdir -p "$HOME/.firecracker"

VERSION=$(echo $ASSET_NAME | grep -oP 'v\d+\.\d+\.\d+')

ARCH=$(uname -m)

cp -r /tmp/release-${VERSION}-${ARCH} $HOME/.firecracker

rm -rf /tmp/release-${VERSION}-${ARCH}

rm $HOME/.firecracker/release-${VERSION}-${ARCH}/firecracker \
   $HOME/.firecracker/release-${VERSION}-${ARCH}/cpu-template-helper \
   $HOME/.firecracker/release-${VERSION}-${ARCH}/jailer \
   $HOME/.firecracker/release-${VERSION}-${ARCH}/rebase-snap \
   $HOME/.firecracker/release-${VERSION}-${ARCH}/seccompiler-bin \
   $HOME/.firecracker/release-${VERSION}-${ARCH}/snapshot-editor

ln -s ${HOME}/.firecracker/release-${VERSION}-${ARCH}/firecracker-${VERSION}-${ARCH} $HOME/.firecracker/release-${VERSION}-${ARCH}/firecracker

ln -s ${HOME}/.firecracker/release-${VERSION}-${ARCH}/cpu-template-helper-${VERSION}-${ARCH} $HOME/.firecracker/release-${VERSION}-${ARCH}/cpu-template-helper

ln -s ${HOME}/.firecracker/release-${VERSION}-${ARCH}/jailer-${VERSION}-${ARCH} $HOME/.firecracker/release-${VERSION}-${ARCH}/jailer

ln -s ${HOME}/.firecracker/release-${VERSION}-${ARCH}/rebase-snap-${VERSION}-${ARCH} $HOME/.firecracker/release-${VERSION}-${ARCH}/rebase-snap


ln -s ${HOME}/.firecracker/release-${VERSION}-${ARCH}/seccompiler-bin-${VERSION}-${ARCH} $HOME/.firecracker/release-${VERSION}-${ARCH}/seccompiler-bin


ln -s ${HOME}/.firecracker/release-${VERSION}-${ARCH}/snapshot-editor-${VERSION}-${ARCH} $HOME/.firecracker/release-${VERSION}-${ARCH}/snapshot-editor

SUDO=""

if command -v sudo >/dev/null 2>&1; then
  SUDO=sudo
fi

$SUDO cp $HOME/.firecracker/release-${VERSION}-${ARCH}/firecracker-${VERSION}-${ARCH} /usr/sbin/firecracker

$SUDO cp $HOME/.firecracker/release-${VERSION}-${ARCH}/jailer-${VERSION}-${ARCH} /usr/local/bin/jailer

$SUDO cp $HOME/.firecracker/release-${VERSION}-${ARCH}/cpu-template-helper-${VERSION}-${ARCH} /usr/local/bin/cpu-template-helper

$SUDO cp $HOME/.firecracker/release-${VERSION}-${ARCH}/rebase-snap-${VERSION}-${ARCH} /usr/local/bin/rebase-snap

$SUDO cp $HOME/.firecracker/release-${VERSION}-${ARCH}/seccompiler-bin-${VERSION}-${ARCH} /usr/local/bin/seccompiler-bin

$SUDO cp $HOME/.firecracker/release-${VERSION}-${ARCH}/snapshot-editor-${VERSION}-${ARCH} /usr/local/bin/snapshot-editor

detect_os

RELEASE_URL="https://api.github.com/repos/tsirysndr/fireup/releases/latest"

DOWNLOAD_URL=$(curl -sSL "$RELEASE_URL" | grep -o "browser_download_url.*fireup-.*$ASSET_NAME\"" | cut -d ' ' -f 2)

DOWNLOAD_URL=`echo $DOWNLOAD_URL | tr -d '\"'`

ASSET_NAME=$(basename $DOWNLOAD_URL)

curl -SL $DOWNLOAD_URL -o /tmp/$ASSET_NAME

tar -xvf /tmp/$ASSET_NAME -C /tmp

chmod a+x /tmp/fireup

$SUDO cp /tmp/fireup /usr/local/bin/fireup
rm -rf /tmp/fireup

cat <<EOF
${ORANGE}
     _______           __  __
    / ____(_)_______  / / / /___
   / /_  / / ___/ _ \/ / / / __ \\
  / __/ / / /  /  __/ /_/ / /_/ /
 /_/   /_/_/   \___/\____/ .___/
                        /_/
${NO_COLOR}
Welcome to Fireup!

${GREEN}https://github.com/tsirysndr/fireup${NO_COLOR}

Please file an issue if you encounter any problems!

===============================================================================

Installation completed! 🎉

You can now run the following command to start using Fireup:
${CYAN}fireup${NO_COLOR}

EOF