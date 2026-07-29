use rand::Rng;

pub fn oui_for_profile(profile: &str) -> Option<[u8; 3]> {
    let oui = match profile {
        "intel" => [0x3C, 0xFD, 0xFE],
        "dell" => [0xB8, 0xCA, 0x3A],
        "cisco" => [0x00, 0x25, 0x83],
        "apple" => [0xA4, 0x83, 0xE7],
        "hp_printer" => [0x3C, 0xD9, 0x2B],
        "samsung" => [0x00, 0x25, 0x67],
        _ => return None,
    };
    Some(oui)
}

pub const PROFILES: &[(&str, &str)] = &[
    ("random", "Random (locally administered)"),
    ("intel", "Intel"),
    ("dell", "Dell"),
    ("cisco", "Cisco"),
    ("apple", "Apple"),
    ("hp_printer", "HP Printer"),
    ("samsung", "Samsung"),
];

pub fn generate(profile: &str) -> String {
    let mut rng = rand::thread_rng();
    let octets: [u8; 6] = match oui_for_profile(profile) {
        Some(oui) => [oui[0], oui[1], oui[2], rng.gen(), rng.gen(), rng.gen()],
        None => {
            let mut first: u8 = rng.gen();
            first = (first & 0b1111_1100) | 0b0000_0010;
            [first, rng.gen(), rng.gen(), rng.gen(), rng.gen(), rng.gen()]
        }
    };
    format_mac(&octets)
}

fn format_mac(o: &[u8; 6]) -> String {
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        o[0], o[1], o[2], o[3], o[4], o[5]
    )
}

pub fn is_valid(mac: &str) -> bool {
    let parts: Vec<&str> = mac.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.bytes().all(|b| b.is_ascii_hexdigit()))
}

#[allow(dead_code)]
pub fn is_locally_administered(mac: &str) -> bool {
    first_octet(mac)
        .map(|b| b & 0b0000_0010 != 0)
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn is_unicast(mac: &str) -> bool {
    first_octet(mac)
        .map(|b| b & 0b0000_0001 == 0)
        .unwrap_or(false)
}

#[allow(dead_code)]
fn first_octet(mac: &str) -> Option<u8> {
    let first = mac.split(':').next()?;
    u8::from_str_radix(first, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_is_valid_locally_administered_unicast() {
        for _ in 0..200 {
            let m = generate("random");
            assert!(is_valid(&m), "invalid mac: {m}");
            assert!(is_locally_administered(&m), "not LAA: {m}");
            assert!(is_unicast(&m), "not unicast: {m}");
        }
    }

    #[test]
    fn vendor_profile_keeps_oui() {
        let m = generate("intel");
        assert!(is_valid(&m));
        assert!(m.to_lowercase().starts_with("3c:fd:fe"));
    }

    #[test]
    fn validation_rejects_garbage() {
        assert!(!is_valid("nope"));
        assert!(!is_valid("aa:bb:cc:dd:ee"));
        assert!(!is_valid("aa:bb:cc:dd:ee:ff:00"));
        assert!(!is_valid("aa:bb:cc:dd:ee:zz"));
        assert!(is_valid("AA:BB:CC:DD:EE:FF"));
    }
}
