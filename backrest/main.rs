use std::collections::HashSet;
use std::env;
use std::fs::File;

use age;
use anyhow::{Result, bail};
use clap::ArgMatches;
use dirs;
use ignore;
use rpassword;
use tar::{Builder,Archive};

mod config;

use secrecy::Secret;

fn main() -> Result<()> {
    let matches = config::get_matches();
    if let Some(matches) = matches.subcommand_matches("backup") {
        backup(&matches)?;
    } else if let Some(matches) = matches.subcommand_matches("restore") {
        restore(&matches)?;
    } else {
        println!("Expected backup or restore command. Nothing to do. Exiting.")
    }
    Ok(())
}

fn get_pw(prompt: &str) -> String {
    let mut pw = String::from("a");
    let mut confirm = String::from("b");
    while pw != confirm {
        pw = rpassword::read_password_from_tty(Some(prompt)).unwrap();
        confirm = rpassword::read_password_from_tty(Some("Confirm: ")).unwrap();
    }

    return pw;
}

fn backup(matches: &ArgMatches) -> Result<()> {
    let mut seen_files: HashSet<String> = HashSet::new();
    let mut seen_patterns: HashSet<String> = HashSet::new();
    let user_home = match dirs::home_dir() {
        Some(dir) => match dir.as_path().to_str() {
            Some(dir) => format!("{}", dir),
            None => bail!("home dir exists but is not a dir"),
        },
        None => match env::var("HOME") {
            Ok(val) => format!("{}", val),
            Err(_) => {
                bail!("no home var and no dirs::home_dir")
            },
        },
    };
    let backup_dir = ensure_backup_dir(matches, &user_home);
    println!("user home is {}", &user_home);

    for raw_pattern in matches.values_of("patterns").expect("needed patterns to do backup") {
        let pattern = raw_pattern.trim_end_matches(|c| c == '/');
        println!("backing up {}", pattern);

        let archive_name = get_archive_name(pattern, &user_home);
        if !seen_patterns.insert(archive_name.clone()) {
            continue;
        }

        let archive_path = format!("{}/{}.tar.age", backup_dir, archive_name);
        let mut file = File::create(archive_path.clone()).expect(&format!("Needed to create {}", archive_path));
        let passphrase = get_pw(&format!("Password for {}: ", archive_path));
        let encryptor = age::Encryptor::with_user_passphrase(Secret::new(passphrase.to_owned()));

        let writer = encryptor.wrap_output(&mut file).unwrap();

        let mut archive = Builder::new(writer);

        for result in ignore::WalkBuilder::new(pattern).hidden(false).build() {
            match result {
                Ok(entry) => {
                    let file_type = entry.file_type();
                    if file_type.is_none() {
                        continue;
                    }
                    let is_dir = file_type.and_then(|x: std::fs::FileType| Some(x.is_dir())).unwrap_or(false);
                    if let Some(file_type) = file_type {
                        if file_type.is_symlink() {
                            eprintln!("skipping symlink {:?}", entry.path().to_str());
                            continue;
                        }
                    }

                    if let Some(file_path) = entry.path().to_str() {
                        if seen_files.contains(file_path) {
                            continue
                        } else {
                            seen_files.insert(file_path.to_string());
                        }
                        let mut place_in_archive = if file_path.starts_with(user_home.as_str()) {
                            file_path.replacen(user_home.as_str(), "", 1)
                        } else {
                            file_path.to_string()
                        };
                        if place_in_archive.starts_with("/") {
                            place_in_archive = place_in_archive.replacen("/", "", 1); 
                        }
                        if is_dir {
                            archive.append_dir(place_in_archive, file_path)
                                .expect(format!("could not add dir {}", pattern).as_str());
                        } else {
                            archive.append_path_with_name(file_path, place_in_archive)
                                .expect(format!("could not add path {}", pattern).as_str());
                        }
                    } else {
                        eprintln!("bad entry {:?}", entry)
                    }
                },
                Err(x) => {
                    eprintln!("failed backing up file {}", x)
                },
            }
        }

        let some_ref = archive.into_inner()?;
        some_ref.finish()?;
    }

    Ok(())
}

fn get_archive_name(pattern: &str, user_home: &str) -> String {
    let archive_name = if pattern.starts_with(user_home) {
        pattern.replacen(user_home, "", 1)
    } else {
        pattern.to_string()
    };

    return archive_name.trim_start_matches(|c| c == '/')
        .replace("/", "_")
        .replace(".", "_")
        .replace(" ", "_")
        .replace("*", "_");
}

fn ensure_backup_dir(matches: &ArgMatches, user_home: &String) -> String {
    if let Some(dir) = matches.value_of("backdir") {
        dir.to_string()
    } else {
        user_home.to_string()
    }
}

fn restore(matches: &ArgMatches) -> Result<()> {
    let dest = matches.value_of("dest").expect("dest is required"); // dest required
    for backup in matches.values_of("backups").expect("needed backups to do restore") {
        let backup_handle = match File::open(backup) {
            Ok(x) => x,
            Err(x) => {
                println!("failed {} backup: {}", backup, x);
                continue
            }
        };
        let d = match age::Decryptor::new(backup_handle) {
            Ok(age::Decryptor::Passphrase(d)) => d,
            _ => {
                println!("failed {} backup: couldn't decrypt as password", backup);
                continue
            }
        };
        if let Err(x) = std::fs::create_dir_all(dest) {
            println!("failed {} backup: {}", backup, x);
            continue
        };
        let passphrase = get_pw(&format!("Password for {}: ", backup));
        let result = match d.decrypt(&Secret::new(passphrase), None) {
            Ok(x) => x,
            Err(x) => {
                println!("failed {} backup: {}", backup, x);
                continue
            },
        };
        if let Err(x) = Archive::new(result).unpack(dest) {
            println!("Unable to unpack archive for {}: {}", backup, x);
        };
    }
    Ok(())
}
