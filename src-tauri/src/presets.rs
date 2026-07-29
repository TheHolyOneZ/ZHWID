use rand::Rng;

pub struct Preset {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub mac_profile: &'static str,
    pub machine_id: bool,
    pub hostname_on: bool,
    pub dhcp: bool,
}

impl Preset {
    pub fn hostname(&self) -> String {
        let mut rng = rand::thread_rng();
        match self.id {
            "printer" => format!("NPI{:06X}", rng.gen::<u32>() & 0x00FF_FFFF),
            "workstation" => {
                let tail: String = (0..7).map(|_| b36(rng.gen_range(0..36))).collect();
                format!("DESKTOP-{tail}")
            }
            "maconly" => String::new(),
            _ => format!("host-{:04x}", rng.gen::<u16>()),
        }
    }
}

fn b36(n: u32) -> char {
    let n = n % 36;
    if n < 10 {
        (b'0' + n as u8) as char
    } else {
        (b'A' + (n - 10) as u8) as char
    }
}

pub const PRESETS: &[Preset] = &[
    Preset {
        id: "balanced",
        label: "Balanced",
        description: "Everyday privacy — random MAC, fresh machine-id, hostname & DHCP identity.",
        mac_profile: "random",
        machine_id: true,
        hostname_on: true,
        dhcp: true,
    },
    Preset {
        id: "printer",
        label: "Pentest · Printer",
        description: "Blend in as an HP network printer (HP OUI + JetDirect-style name).",
        mac_profile: "hp_printer",
        machine_id: true,
        hostname_on: true,
        dhcp: true,
    },
    Preset {
        id: "workstation",
        label: "Pentest · Workstation",
        description: "Look like a Windows workstation (Dell OUI + DESKTOP-xxxx name).",
        mac_profile: "dell",
        machine_id: true,
        hostname_on: true,
        dhcp: true,
    },
    Preset {
        id: "maconly",
        label: "MAC only",
        description: "Minimal footprint — spoof just the network MAC (and DHCP).",
        mac_profile: "random",
        machine_id: false,
        hostname_on: false,
        dhcp: true,
    },
];

pub fn by_id(id: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.id == id)
}
