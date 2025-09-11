use rand::Rng;

pub fn generate_unique_mac() -> String {
    let mut rng = rand::thread_rng();

    // First byte must have the least significant bit set to 0 (unicast)
    // and the second least significant bit set to 0 (globally unique)
    let first_byte = (rng.gen::<u8>() & 0xFC) | 0x02; // Set locally administered bit

    // Generate the remaining 5 bytes
    let mut mac_bytes = [0u8; 6];
    mac_bytes[0] = first_byte;
    for i in 1..6 {
        mac_bytes[i] = rng.gen::<u8>();
    }

    // Format as a MAC address (XX:XX:XX:XX:XX:XX)
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac_bytes[0], mac_bytes[1], mac_bytes[2], mac_bytes[3], mac_bytes[4], mac_bytes[5]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_unique_mac() {
        let mac1 = generate_unique_mac();
        let mac2 = generate_unique_mac();

        // Check format (6 pairs of 2 hex digits separated by colons)
        assert!(mac1.chars().count() == 17);
        assert!(mac1.split(':').count() == 6);

        // Check that the MAC is locally administered (second bit of first byte is 1)
        let first_byte = u8::from_str_radix(&mac1[0..2], 16).unwrap();
        assert_eq!(first_byte & 0x02, 0x02);
        assert_eq!(first_byte & 0x01, 0x00); // Unicast

        // Check uniqueness
        assert_ne!(mac1, mac2);
    }
}
