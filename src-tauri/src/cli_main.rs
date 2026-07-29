use clap::{Args, Parser, Subcommand};
use z_hwid_lib::privileged::{IfaceSpoof, SpoofConfig};
use z_hwid_lib::{dhcp, engine, generate, identity, mac, presets, privileged, profile, state};

#[derive(Parser)]
#[command(
    name = "zhwid",
    version,
    about = "Z-HWID — Linux identity switch (CLI)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Status,

    On(OnArgs),

    #[command(alias = "restore")]
    Off,

    Presets,

    Cloak(CloakArgs),

    Gen(GenArgs),
}

#[derive(Args)]
struct OnArgs {
    #[arg(short, long, default_value = "balanced")]
    preset: String,

    #[arg(long)]
    pre_connect: bool,

    #[arg(long, value_delimiter = ',')]
    iface: Vec<String>,

    #[arg(long)]
    no_machine_id: bool,

    #[arg(long)]
    no_hostname: bool,

    #[arg(long)]
    no_dhcp: bool,

    #[arg(long)]
    dry_run: bool,
}

#[derive(Args)]
struct CloakArgs {
    app: String,
}

#[derive(Args)]
struct GenArgs {
    what: String,

    #[arg(long, default_value = "random")]
    profile: String,
}

fn main() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    std::process::exit(match Cli::parse().cmd {
        Cmd::Status => cmd_status(),
        Cmd::On(a) => cmd_on(a),
        Cmd::Off => cmd_off(),
        Cmd::Presets => cmd_presets(),
        Cmd::Cloak(a) => cmd_cloak(a),
        Cmd::Gen(a) => cmd_gen(a),
    });
}

fn build_config(a: &OnArgs) -> Result<(SpoofConfig, &'static presets::Preset), String> {
    let preset = presets::by_id(&a.preset)
        .ok_or_else(|| format!("unknown preset '{}' — run `zhwid presets`", a.preset))?;

    let mut ifaces = identity::list_interfaces();
    if !a.iface.is_empty() {
        ifaces.retain(|i| a.iface.iter().any(|n| n == &i.name));
        if ifaces.is_empty() {
            return Err("no matching interfaces".to_string());
        }
    }
    let interfaces = ifaces
        .iter()
        .map(|i| IfaceSpoof {
            name: i.name.clone(),
            mac: mac::generate(preset.mac_profile),
        })
        .collect();

    Ok((
        SpoofConfig {
            interfaces,
            set_machine_id: preset.machine_id && !a.no_machine_id,
            machine_id: generate::machine_id(),
            set_hostname: preset.hostname_on && !a.no_hostname,
            hostname: preset.hostname(),
            pre_connect: a.pre_connect,
            spoof_dhcp: preset.dhcp && !a.no_dhcp && dhcp::nm_available(),
        },
        preset,
    ))
}

fn cmd_on(a: OnArgs) -> i32 {
    let (config, preset) = match build_config(&a) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };

    if a.dry_run {
        let backup = privileged::capture_backup(&config);
        return match privileged::build_apply_script(&config, &backup) {
            Ok(script) => {
                println!("# preset: {} ({})", preset.label, preset.id);
                println!("# would run as root:\n{script}");
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                2
            }
        };
    }

    println!(
        "Applying preset '{}' — you'll be asked to authenticate…",
        preset.label
    );
    match engine::apply(&config) {
        Ok(out) => {
            if !out.is_empty() {
                println!("{out}");
            }
            println!("✓ spoof applied\n");
            cmd_status()
        }
        Err(e) => {
            eprintln!("✗ {e}");
            1
        }
    }
}

fn cmd_off() -> i32 {
    println!("Restoring original identity — you'll be asked to authenticate…");
    match engine::restore() {
        Ok(out) => {
            if !out.is_empty() {
                println!("{out}");
            }
            println!("✓ restored\n");
            cmd_status()
        }
        Err(e) => {
            eprintln!("✗ {e}");
            1
        }
    }
}

fn cmd_status() -> i32 {
    let active = state::is_active();
    let st = identity::full_status(active);
    let backup = state::load();

    println!(
        "Z-HWID: {}",
        if active {
            "● SPOOFED (ON)"
        } else {
            "○ real (OFF)"
        }
    );

    println!("\nNetwork (MAC):");
    for i in &st.interfaces {
        let tag = if i.spoofed { "spoofed" } else { "real" };
        println!("  {:<8} {}  [{}]", i.name, i.current_mac, tag);
        if i.spoofed {
            println!("  {:<8} real {}", "", i.perm_mac);
        }
    }

    println!("\nSystem:");
    field("machine-id", &st.machine_id, backup.machine_id.as_deref());
    field("hostname", &st.hostname, backup.hostname.as_deref());

    if dhcp::nm_available() {
        let by_name: std::collections::HashMap<_, _> = backup
            .interfaces
            .iter()
            .map(|i| (i.name.clone(), i))
            .collect();
        println!("\nDHCP identity:");
        for i in &st.interfaces {
            if let Some(conn) = dhcp::connection_for(&i.name) {
                let (cid, _duid) = dhcp::read_dhcp(&conn);
                let orig = by_name.get(&i.name).and_then(|b| b.dhcp_client_id.clone());
                let tag = match &orig {
                    Some(o) if o != &cid => "spoofed",
                    Some(_) => "real",
                    None => "?",
                };
                let shown = if cid.is_empty() { "(default)" } else { &cid };
                println!("  {:<8} client-id {}  [{}]", i.name, shown, tag);
            }
        }
    }
    0
}

fn field(name: &str, current: &str, original: Option<&str>) {
    let tag = match original {
        Some(o) if o != current => "spoofed",
        Some(_) => "real",
        None => "?",
    };
    println!("  {:<12} {}  [{}]", name, current, tag);
    if matches!(original, Some(o) if o != current) {
        println!("  {:<12} real {}", "", original.unwrap());
    }
}

fn cmd_presets() -> i32 {
    println!("Presets:");
    for p in presets::PRESETS {
        println!("  {:<12} {}", p.id, p.label);
        println!("  {:<12} {}", "", p.description);
    }
    0
}

fn cmd_cloak(a: CloakArgs) -> i32 {
    let prof = profile::HardwareProfile::random();
    match z_hwid_lib::cloak::launch(&a.app, &prof) {
        Ok(msg) => {
            println!("{msg}");
            0
        }
        Err(e) => {
            eprintln!("✗ {e}");
            1
        }
    }
}

fn cmd_gen(a: GenArgs) -> i32 {
    match a.what.as_str() {
        "mac" => println!("{}", mac::generate(&a.profile)),
        "machine-id" => println!("{}", generate::machine_id()),
        "hostname" => println!("{}", generate::hostname()),
        other => {
            eprintln!("unknown target '{other}' (use: mac | machine-id | hostname)");
            return 2;
        }
    }
    0
}
