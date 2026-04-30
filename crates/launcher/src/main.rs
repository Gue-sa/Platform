use colored::{ColoredString, Colorize};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use shared::config::Config;
use std::{
    fs,
    io::{Error, Write},
    net::{IpAddr, Ipv4Addr},
    process::{self, Command, ExitStatus, Stdio},
    str::FromStr,
};
use sudo::RunningAs;

const VHOSTS_SETUP_SCRIPT: &str = include_str!("../setup_vhosts.sh");

fn setup_vhosts() -> () {
    let mut child = Command::new("bash")
        .arg("-s") // Lit le script depuis stdin
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit()) // Affiche les sorties du script dans ton terminal
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().expect("Échec ouverture stdin");
    stdin.write_all(VHOSTS_SETUP_SCRIPT.as_bytes());
    drop(stdin); // Important : ferme le pipe pour que bash commence l'exécution

    child.wait();
}

fn build_config() -> () {
    let mut config: Config = Config::default();

    let is_sim: bool = Select::with_theme(&ColorfulTheme::default())
        .default(0)
        .with_prompt("\nVoulez-vous créer une simulation locale (sur une seule machine) ou mettre en place la maquette réelle ?")
        .items(&["Simulation (recquiert sudo pour setup les vhosts + simulation à 1 bateau max)", "Maquette"])
        .interact()
        .unwrap() == 0;

    if is_sim {
        config.set_is_simulation(true);
        config.set_server_ip(IpAddr::V4(Ipv4Addr::from_str("10.0.0.2").unwrap()));
        config.set_harbourmaster_ip(IpAddr::V4(Ipv4Addr::from_str("10.0.0.1").unwrap()));

        setup_vhosts();
    } else {
        let server_ip: String = Input::new()
            .with_prompt("\nVeuillez entrer l'IPv4 du serveur")
            .validate_with(|val: &String| -> Result<(), &str> {
                if val.parse::<IpAddr>().is_ok() {
                    Ok(())
                } else {
                    Err("Format d'IPv4 invalide")
                }
            })
            .interact_text()
            .unwrap();

        let harbourmaster_ip: String = Input::new()
            .with_prompt("\nVeuillez entrer l'IPv4 de la capitainerie")
            .validate_with(|val: &String| -> Result<(), &str> {
                if val.parse::<IpAddr>().is_ok() {
                    Ok(())
                } else {
                    Err("Format d'IPv4 invalide")
                }
            })
            .interact_text()
            .unwrap();

        config.set_server_ip(server_ip.parse::<IpAddr>().unwrap());
        config.set_harbourmaster_ip(harbourmaster_ip.parse::<IpAddr>().unwrap());
    }

    config.write();

    println!("\nConfiguration effectuée avec succès !\n")
}

fn main() {
    ctrlc::set_handler(move || {}).expect("Erreur lors de la définition du gestionnaire Ctrl+C");

    if sudo::check() == RunningAs::Root {
        println!("Ce simulateur doit être lancé avec sudo.");

        sudo::escalate_if_needed().expect("Erreur lors de l'élévation de privilèges.");
    }

    let functionalities: [&str; 8] = [
        "Déployer la capitainerie (simulation ou montage réel)",
        "Déployer le serveur (simulation)",
        "Déployer un bateau (simulation)",
        "Déployer le serveur (montage réel)",
        "Déployer un bateau (montage réel)",
        "Relancer la configuration",
        "Réinitialiser la base de données de la capitainerie (conseillé avant chaque lancement)",
        "Quitter",
    ];

    let banner: &str = "
    ____  __      __       ____                             __  ___           _ __  _              
   / __ \\/ /___ _/ /____  / __/___  _________ ___  ___     /  |/  /___ ______(_) /_(_)___ ___  ___ 
  / /_/ / / __ `/ __/ _ \\/ /_/ __ \\/ ___/ __ `__ \\/ _ \\   / /|_/ / __ `/ ___/ / __/ / __ `__ \\/ _ \\
 / ____/ / /_/ / /_/  __/ __/ /_/ / /  / / / / / /  __/  / /  / / /_/ / /  / / /_/ / / / / / /  __/
/_/   /_/\\__,_/\\__/\\___/_/  \\____/_/  /_/ /_/ /_/\\___/  /_/  /_/\\__,_/_/  /_/\\__/_/_/ /_/ /_/\\___/                                                                                    
";

    let msg: ColoredString = format!("{banner}\n\n##################################################################################################\n\nVersion 1.0.0\nEcole Nationale Supérieure des Mines de Nancy\nCampus ARTEM et de Saint-Dié-des-Vosges\nUniversité de Lorraine\n2026\n\n##################################################################################################\n\nRéalisé par:\n- Sasha Guérin--Loison (code)\n- Alexandre Brisset (communication VHF, modélisation, fabrication)\n- Saad Ouadrassi (code et algorithme de déplacement)\n- Matieu Gauthier (modélisation, fabrication)\n- Bosco Perrin (conception et fabrication des bateaux)\n- Yasmine ? (conception et fabrication des bateaux)\n\n##################################################################################################\n\nEncadré par:\n- Guillaume Bonfante\n\n##################################################################################################\n\n").yellow();

    Command::new("clear")
        .status()
        .expect("\nImpossible de clear le terminal.\n");

    println!("{msg}");

    if Config::load().is_none() {
        println!("Fichier de configuration non trouvé. Lancement du formulaire de création.\n");

        build_config();
    }

    while let Ok(choice) = Select::with_theme(&ColorfulTheme::default())
        .default(0)
        .with_prompt("\nVeuillez choisir l'action à effectuer")
        .items(functionalities)
        .interact()
    {
        match choice {
            0 => {
                if fs::exists("harbourmaster").is_ok() {
                    if !*Config::load().unwrap().is_simulation() {
                        println!(
                            "\nAttention, la configuration pour simulation n'a pas été effectuée. Le programme va se lancer, mais ne fonctionnera pas en simulation locale.\n"
                        )
                    }

                    println!("\nLancement de la capitainerie...\n");

                    Command::new("./harbourmaster")
                        .status()
                        .expect("Le lancement de la capitainerie a échoué");
                } else {
                    eprintln!(
                        "Impossible de trouver l'exécutable harbourmaster. Demandez-le à Sasha."
                    )
                }
            }
            1 => {
                if *Config::load().unwrap().is_simulation() {
                    if fs::exists("server").is_ok() {
                        println!("\nLancement du serveur...\n");

                        Command::new("sudo")
                            .args(["ip", "netns", "exec", "server", "./server"])
                            .status()
                            .expect("Le lancement du serveur a échoué");
                    } else {
                        eprintln!("Impossible de trouver l'exécutable server. Demandez-le à Sasha.")
                    }
                } else {
                    eprintln!(
                        "La configuration simulation n'a pas été effectuée ! Veuillez y procéder."
                    )
                }
            }
            2 => {
                if *Config::load().unwrap().is_simulation() {
                    if fs::exists("boat").is_ok() {
                        println!("\nLancement du bateau...\n");

                        Command::new("sudo")
                            .args(["ip", "netns", "exec", "boat", "./boat"])
                            .status()
                            .expect("Le lancement du bateau a échoué");
                    } else {
                        eprintln!("Impossible de trouver l'exécutable boat. Demandez-le à Sasha.")
                    }
                } else {
                    eprintln!(
                        "La configuration simulation n'a pas été effectuée ! Veuillez y procéder."
                    )
                }
            }
            3 => {
                if !*Config::load().unwrap().is_simulation() {
                    if fs::exists("server").is_ok() {
                        println!("\nLancement du serveur...\n");

                        Command::new("./server")
                            .status()
                            .expect("Le lancement du serveur a échoué");
                    } else {
                        eprintln!("Impossible de trouver l'exécutable server. Demandez-le à Sasha.")
                    }
                } else {
                    eprintln!(
                        "La configuration maquette réelle n'a pas été effectuée ! Veuillez y procéder."
                    )
                }
            }
            4 => {
                if !*Config::load().unwrap().is_simulation() {
                    if fs::exists("boat").is_ok() {
                        println!("\nLancement du bateau...\n");

                        Command::new("./boat")
                            .status()
                            .expect("Le lancement du bateau a échoué");
                    } else {
                        eprintln!("Impossible de trouver l'exécutable boat. Demandez-le à Sasha.")
                    }
                } else {
                    eprintln!(
                        "La configuration maquette réelle n'a pas été effectuée ! Veuillez y procéder."
                    )
                }
            }
            5 => {
                build_config();
            }
            6 => {
                if Select::with_theme(&ColorfulTheme::default())
                    .default(0)
                    .with_prompt("\nEn êtes-vous sûr ? Cette action est irréversible")
                    .items(&["Non", "Oui"])
                    .interact()
                    .unwrap()
                    == 1
                {
                    fs::remove_file("./harbourmaster_database.db");

                    println!("DB de la capitainerie supprimée avec succès.");
                }
            }
            _ => {
                break;
            }
        }
    }
}
