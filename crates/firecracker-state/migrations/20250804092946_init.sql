-- Add migration script here
CREATE TABLE IF NOT EXISTS virtual_machines (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    status VARCHAR(255)  NOT NULL,
    vcpu INT NOT NULL,
    memory INT NOT NULL,
    distro VARCHAR(255) NOT NULL,
    pid INT,
    mac_address VARCHAR(255) NOT NULL,
    bridge VARCHAR(255) NOT NULL,
    tap VARCHAR(255) NOT NULL,
    api_socket VARCHAR(255) UNIQUE NOT NULL,
    project_dir VARCHAR(255) UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
