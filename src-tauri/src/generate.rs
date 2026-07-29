use rand::seq::SliceRandom;
use rand::Rng;

pub fn machine_id() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| std::char::from_digit(rng.gen_range(0..16), 16).unwrap())
        .collect()
}

pub fn uuid() -> String {
    let mut rng = rand::thread_rng();
    let mut b = [0u8; 16];
    rng.fill(&mut b);
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]
    )
}

pub fn serial() -> String {
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..10)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

const HOST_WORDS: &[&str] = &[
    "desktop",
    "laptop",
    "workstation",
    "thinkpad",
    "pc",
    "notebook",
    "home",
    "studio",
];

pub fn hostname() -> String {
    let mut rng = rand::thread_rng();
    let word = HOST_WORDS.choose(&mut rng).copied().unwrap_or("host");
    format!("{word}-{:04x}", rng.gen::<u16>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_id_is_32_hex() {
        let id = machine_id();
        assert_eq!(id.len(), 32);
        assert!(id.bytes().all(|b| b.is_ascii_hexdigit()));
    }

    #[test]
    fn hostname_is_dns_safe() {
        let h = hostname();
        assert!(h.len() <= 63);
        assert!(h.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'));
    }
}
