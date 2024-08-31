use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};

fn main() {
    // Farben definieren
    let yellow = "\x1b[1;33m";
    let red = "\x1b[0;31m";
    let green = "\x1b[0;32m";
    let nc = "\x1b[0m";

    // Überprüfen, ob das Skript mit Root-Rechten ausgeführt wird
    if env::var("USER").unwrap_or_default() != "root" {
        eprintln!("{}WARNING: {}Please run with sudo privileges.", yellow, nc);
        exit(1);
    }

    // Überprüfen, ob das Betriebssystem Ubuntu ist
    let os_release = fs::read_to_string("/etc/os-release").unwrap_or_default();
    if !os_release.contains("ID=ubuntu") {
        eprintln!("{}ERROR: {}You are trying to install this on an unsupported distribution.\nOnly Ubuntu is supported.", red, nc);
        exit(1);
    }

    // Snapd deinstallieren
    println!("{}Uninstall snapd...{}", red, nc);
    let preferences_dir = "/etc/apt/preferences.d/";
    if !Path::new(preferences_dir).exists() {
        fs::create_dir_all(preferences_dir).unwrap();
    }

    let nosnap_path = format!("{}nosnap.pref", preferences_dir);
    let mut file = File::create(&nosnap_path).unwrap();
    writeln!(file, "# To prevent repository packages from triggering the installation of snap,\n# this file forbids snapd from being installed by APT.\n\nPackage: snapd\nPin: release a=*\nPin-Priority: -10").unwrap();

    // Snap-Pakete entfernen
    let snap_list_output = Command::new("snap")
        .arg("list")
        .output()
        .expect("Failed to list snap packages");

    let snap_list_output_str = String::from_utf8_lossy(&snap_list_output.stdout);

    let snaps: Vec<&str> = snap_list_output_str
        .lines()
        .skip(1) // Überspringt die Kopfzeile
        .filter(|line| !line.starts_with("core")) // Filtert 'core' aus
        .map(|line| line.split_whitespace().next().unwrap()) // Extrahiert den Paketnamen
        .collect();

    for snap in snaps {
        let _ = Command::new("snap")
            .arg("remove")
            .arg("--purge")
            .arg(snap)
            .output();
    }

    // Snapd und zugehörige Pakete entfernen
    let _ = Command::new("apt-get")
        .arg("purge")
        .arg("-y")
        .arg("snapd")
        .arg("gnome-software-plugin-snap")
        .output();

    let paths_to_remove = [
        "~/snap",
        "/snap",
        "/var/snap",
        "/var/lib/snapd",
        "/var/cache/snapd",
    ];

    for path in paths_to_remove.iter() {
        let _ = fs::remove_dir_all(path);
    }

    let _ = Command::new("systemctl").arg("daemon-reload").output();

    // Flatpak installieren
    println!("{}Install Flatpak...{}", green, nc);
    let _ = Command::new("add-apt-repository")
        .arg("ppa:flatpak/stable")
        .output();

    let _ = Command::new("apt-get").arg("update").output();

    let _ = Command::new("apt-get")
        .arg("install")
        .arg("-y")
        .arg("flatpak")
        .arg("gnome-software-plugin-flatpak")
        .output();

    let _ = Command::new("flatpak")
        .arg("remote-add")
        .arg("--if-not-exists")
        .arg("flathub")
        .arg("https://flathub.org/repo/flathub.flatpakrepo")
        .output();

    println!("It's recommended to reboot the system! 'sudo systemctl reboot'");
}
