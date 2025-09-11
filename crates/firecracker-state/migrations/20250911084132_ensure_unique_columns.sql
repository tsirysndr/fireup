-- Add migration script here
CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_mac_address ON virtual_machines (mac_address);
CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_pid ON virtual_machines (pid);
CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_tap ON virtual_machines (tap);