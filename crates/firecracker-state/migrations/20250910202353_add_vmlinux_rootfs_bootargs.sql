-- Add migration script here
ALTER TABLE virtual_machines
ADD COLUMN vmlinux VARCHAR(255);

ALTER TABLE virtual_machines
ADD COLUMN rootfs VARCHAR(255);

ALTER TABLE virtual_machines
ADD COLUMN bootargs TEXT;