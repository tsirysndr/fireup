-- Add migration script here
ALTER TABLE virtual_machines
ADD COLUMN ssh_keys TEXT;