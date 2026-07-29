use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::generate;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HardwareProfile {
    pub sys_vendor: String,
    pub product_name: String,
    pub product_version: String,
    pub product_uuid: String,
    pub product_serial: String,
    pub board_vendor: String,
    pub board_name: String,
    pub board_serial: String,
    pub bios_vendor: String,
    pub bios_version: String,
    pub chassis_serial: String,
    pub machine_id: String,
    pub disk_serial: String,
}

struct Vendor {
    sys: &'static str,
    board_vendor: &'static str,
    bios_vendor: &'static str,
    products: &'static [&'static str],
    boards: &'static [&'static str],
}

const VENDORS: &[Vendor] = &[
    Vendor {
        sys: "Dell Inc.",
        board_vendor: "Dell Inc.",
        bios_vendor: "Dell Inc.",
        products: &[
            "XPS 15 9520",
            "Latitude 7430",
            "OptiPlex 7090",
            "Inspiron 15 3520",
        ],
        boards: &["0G2D9F", "0R6JMH", "0K3WM9"],
    },
    Vendor {
        sys: "LENOVO",
        board_vendor: "LENOVO",
        bios_vendor: "LENOVO",
        products: &[
            "ThinkPad X1 Carbon Gen 10",
            "ThinkPad T14 Gen 3",
            "IdeaPad 5 Pro",
        ],
        boards: &["21CB", "21AH", "LNVNB161216"],
    },
    Vendor {
        sys: "ASUSTeK COMPUTER INC.",
        board_vendor: "ASUSTeK COMPUTER INC.",
        bios_vendor: "American Megatrends Inc.",
        products: &["ROG Strix G15", "ZenBook UX425EA", "TUF Gaming A15"],
        boards: &[
            "ROG STRIX B550-F GAMING",
            "PRIME X570-P",
            "TUF GAMING B650-PLUS",
        ],
    },
    Vendor {
        sys: "HP",
        board_vendor: "HP",
        bios_vendor: "HP",
        products: &["HP EliteBook 840 G8", "HP Pavilion 15", "HP ProBook 450 G9"],
        boards: &["8846", "87D6", "8A78"],
    },
    Vendor {
        sys: "Micro-Star International Co., Ltd.",
        board_vendor: "Micro-Star International Co., Ltd.",
        bios_vendor: "American Megatrends International, LLC.",
        products: &["MS-7C56", "Katana GF66", "Prestige 14"],
        boards: &["B550-A PRO (MS-7C56)", "MPG B650 EDGE WIFI (MS-7E10)"],
    },
];

impl HardwareProfile {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let v = VENDORS.choose(&mut rng).unwrap_or(&VENDORS[0]);
        let bios_version = format!(
            "{}.{}.{}",
            rng.gen_range(1..3),
            rng.gen_range(0..30),
            rng.gen_range(0..10)
        );
        HardwareProfile {
            sys_vendor: v.sys.to_string(),
            product_name: v.products.choose(&mut rng).unwrap().to_string(),
            product_version: format!("{}.0", rng.gen_range(1..4)),
            product_uuid: generate::uuid(),
            product_serial: generate::serial(),
            board_vendor: v.board_vendor.to_string(),
            board_name: v.boards.choose(&mut rng).unwrap().to_string(),
            board_serial: generate::serial(),
            bios_vendor: v.bios_vendor.to_string(),
            bios_version,
            chassis_serial: generate::serial(),
            machine_id: generate::machine_id(),
            disk_serial: generate::serial(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_is_coherent() {
        let p = HardwareProfile::random();
        assert!(!p.sys_vendor.is_empty());
        assert!(!p.product_name.is_empty());
        assert_eq!(p.machine_id.len(), 32);
        assert!(p.product_uuid.contains('-'));
    }
}
