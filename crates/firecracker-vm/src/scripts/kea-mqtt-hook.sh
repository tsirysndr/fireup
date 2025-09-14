#!/usr/bin/env bash
set -euo pipefail

export BROKER="localhost"
export PORT="1883"
export TOPIC="/dhcp/hwaddr"

echo "Starting Kea DHCP hook script..."
echo $1 $QUERY4_HWADDR

if [ "$1" = "leases4_committed" ]; then
  if [ -z "${QUERY4_HWADDR:-}" ]; then
    echo "QUERY4_HWADDR is not set. Exiting."
    exit 0
  fi

  echo ">> Lease committed event detected." >> /tmp/kea-lease-hook.log
  env >> /tmp/kea-lease-hook.log
  # Log to a file
  echo "$(date): New lease assigned - IP: $LEASES4_AT0_ADDRESS, MAC: $QUERY4_HWADDR" >> /tmp/kea-lease-hook.log

  echo "New lease assigned - IP: $LEASES4_AT0_ADDRESS, MAC: $QUERY4_HWADDR"

  mosquitto_pub -h "$BROKER" -p "$PORT" -t "$TOPIC" -m "REQUEST $QUERY4_HWADDR $LEASES4_AT0_ADDRESS binding"
fi

exit 0
