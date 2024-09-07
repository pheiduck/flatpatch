// Copyright (c) Philip H.

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
    let user = env::var("USER").unwrap_or_default();
    if user != "root" {
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
    println!("{}Uninstalling snapd...{}", red, nc);
    let preferences_dir = "/etc/apt/preferences.d/";
    if !Path::new(preferences_dir).exists() {
        println!("{}Creating directory: {}{}", green, preferences_dir, nc);
        fs::create_dir_all(preferences_dir).unwrap();
    }

    let nosnap_path = format!("{}nosnap.pref", preferences_dir);
    println!("{}Creating file: {}{}", green, nosnap_path, nc);
    let mut file = File::create(&nosnap_path).unwrap();
    writeln!(file, "# To prevent repository packages from triggering the installation of snap,\n# this file forbids snapd from being installed by APT.\n\nPackage: snapd\nPin: release a=*\nPin-Priority: -10").unwrap();

    // Snap-Pakete entfernen
    println!("{}Listing snap packages...{}", green, nc);
    let snap_list_output = Command::new("snap")
        .arg("list")
        .output()
        .expect("Failed to list snap packages");

    let snap_list_output_str = String::from_utf8_lossy(&snap_list_output.stdout);
    println!("{}Installed snaps:\n{}{}", green, snap_list_output_str, nc);

    let snaps: Vec<&str> = snap_list_output_str
        .lines()
        .skip(1) // Überspringt die Kopfzeile
        .filter(|line| !line.starts_with("core")) // Filtert 'core' aus
        .map(|line| line.split_whitespace().next().unwrap()) // Extrahiert den Paketnamen
        .collect();

    for snap in &snaps {
        println!("{}Removing snap: {}{}", red, snap, nc);
        let output = Command::new("snap")
            .arg("remove")
            .arg("--purge")
            .arg(snap)
            .output()
            .unwrap_or_else(|_| panic!("Failed to remove snap package: {}", snap));

        if !output.status.success() {
            eprintln!(
                "{}Error removing snap '{}':\n{}{}",
                red,
                snap,
                String::from_utf8_lossy(&output.stderr),
                nc
            );
        } else {
            println!("{}Successfully removed snap '{}'.{}", green, snap, nc);
        }
    }

    // Snapd und zugehörige Pakete entfernen
    println!("{}Purging snapd and related packages...{}", red, nc);
    let output = Command::new("apt-get")
        .arg("purge")
        .arg("-y")
        .arg("snapd")
        .arg("gnome-software-plugin-snap")
        .output()
        .expect("Failed to purge snapd and related packages");

    if !output.status.success() {
        eprintln!(
            "{}Error purging snapd:\n{}{}",
            red,
            String::from_utf8_lossy(&output.stderr),
            nc
        );
    } else {
        println!(
            "{}Successfully purged snapd and related packages.{}",
            green, nc
        );
    }

    // Verzeichnisse entfernen
    let paths_to_remove = [
        "~/snap",
        "/snap",
        "/var/snap",
        "/var/lib/snapd",
        "/var/cache/snapd",
    ];

    for path in &paths_to_remove {
        if Path::new(path).exists() {
            println!("{}Removing directory: {}{}", red, path, nc);
            let result = fs::remove_dir_all(path);
            match result {
                Ok(_) => println!("{}Successfully removed directory: {}{}", green, path, nc),
                Err(e) => eprintln!("{}Error removing directory '{}': {}{}", red, path, e, nc),
            }
        } else {
            println!("{}Directory does not exist: {}{}", yellow, path, nc);
        }
    }

    println!("{}Reloading systemd daemon...{}", green, nc);
    let output = Command::new("systemctl")
        .arg("daemon-reload")
        .output()
        .expect("Failed to reload systemd daemon");
    if !output.status.success() {
        eprintln!(
            "{}Error reloading systemd daemon:\n{}{}",
            red,
            String::from_utf8_lossy(&output.stderr),
            nc
        );
    } else {
        println!("{}Successfully reloaded systemd daemon.{}", green, nc);
    }

    // Flatpak installieren
    println!("{}Installing Flatpak...{}", green, nc);
    let output = Command::new("add-apt-repository")
        .arg("ppa:flatpak/stable")
        .output()
        .expect("Failed to add Flatpak PPA");

    if !output.status.success() {
        eprintln!(
            "{}Error adding Flatpak PPA:\n{}{}",
            red,
            String::from_utf8_lossy(&output.stderr),
            nc
        );
    } else {
        println!("{}Successfully added Flatpak PPA.{}", green, nc);
    }

    let output = Command::new("apt-get")
        .arg("update")
        .output()
        .expect("Failed to update package list");
    if !output.status.success() {
        eprintln!(
            "{}Error updating package list:\n{}{}",
            red,
            String::from_utf8_lossy(&output.stderr),
            nc
        );
    } else {
        println!("{}Successfully updated package list.{}", green, nc);
    }

    let output = Command::new("apt-get")
        .arg("install")
        .arg("-y")
        .arg("flatpak")
        .arg("gnome-software-plugin-flatpak")
        .output()
        .expect("Failed to install Flatpak and plugin");

    if !output.status.success() {
        eprintln!(
            "{}Error installing Flatpak:\n{}{}",
            red,
            String::from_utf8_lossy(&output.stderr),
            nc
        );
    } else {
        println!("{}Successfully installed Flatpak and plugin.{}", green, nc);
    }

    let output = Command::new("flatpak")
        .arg("remote-add")
        .arg("--if-not-exists")
        .arg("flathub")
        .arg("https://flathub.org/repo/flathub.flatpakrepo")
        .output()
        .expect("Failed to add Flathub remote");

    if !output.status.success() {
        eprintln!(
            "{}Error adding Flathub remote:\n{}{}",
            red,
            String::from_utf8_lossy(&output.stderr),
            nc
        );
    } else {
        println!("{}Successfully added Flathub remote.{}", green, nc);
    }

    println!(
        "{}It's recommended to reboot the system! 'sudo systemctl reboot'{}",
        yellow, nc
    );
}
