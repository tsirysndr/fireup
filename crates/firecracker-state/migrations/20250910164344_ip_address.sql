-- Add migration script here
ALTER TABLE virtual_machines
ADD COLUMN ip_address VARCHAR(255);