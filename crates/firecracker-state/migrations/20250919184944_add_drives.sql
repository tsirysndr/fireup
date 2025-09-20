-- Add migration script here
CREATE TABLE IF NOT EXISTS drives (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    vm_id VARCHAR(255),
    path_on_host VARCHAR(255) NOT NULL UNIQUE,
    is_root_device BOOLEAN NOT NULL,
    is_read_only BOOLEAN NOT NULL,
    size_in_gb INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (vm_id) REFERENCES virtual_machines(id) ON DELETE CASCADE,
    UNIQUE(vm_id, name)
);