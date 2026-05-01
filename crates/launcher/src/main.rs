use colored::{ColoredString, Colorize};
use dialoguer::{Input, Select, theme::ColorfulTheme};
use num_traits::PrimInt;
use shared::config::Config;
use std::{
    fmt::{Binary, Display},
    fs::{self, File, exists},
    io::{Error, Write},
    net::{IpAddr, Ipv4Addr},
    path::Path,
    process::{Command, Stdio},
    str::FromStr,
};
use sudo::RunningAs;

const VHOSTS_SETUP_SCRIPT: &str = include_str!("../setup_vhosts.sh");
const FUNCTIONALITIES: [&str; 10] = [
    "Déployer la capitainerie (simulation ou montage réel)",
    "Déployer le serveur (simulation)",
    "Déployer un bateau (simulation)",
    "Déployer le serveur (montage réel)",
    "Déployer un bateau (montage réel)",
    "Relancer la configuration",
    "Réinitialiser la base de données de la capitainerie (conseillé avant chaque lancement)",
    "Vider les logs",
    "Paramètres",
    "Quitter",
];

const BANNER_TITLE: &str = "
    ____  __      __       ____                             __  ___           _ __  _              
   / __ \\/ /___ _/ /____  / __/___  _________ ___  ___     /  |/  /___ ______(_) /_(_)___ ___  ___ 
  / /_/ / / __ `/ __/ _ \\/ /_/ __ \\/ ___/ __ `__ \\/ _ \\   / /|_/ / __ `/ ___/ / __/ / __ `__ \\/ _ \\
 / ____/ / /_/ / /_/  __/ __/ /_/ / /  / / / / / /  __/  / /  / / /_/ / /  / / /_/ / / / / / /  __/
/_/   /_/\\__,_/\\__/\\___/_/  \\____/_/  /_/ /_/ /_/\\___/  /_/  /_/\\__,_/_/  /_/\\__/_/_/ /_/ /_/\\___/                                                                                    
";

fn clear_terminal() -> () {
    Command::new("clear")
        .status()
        .expect("\nImpossible de clear le terminal.\n");
}

fn display_banner() -> () {
    let banner_msg: ColoredString = format!("{BANNER_TITLE}\n\n##################################################################################################\n\nVersion 1.0.0\nEcole Nationale Supérieure des Mines de Nancy\nCampus ARTEM et de Saint-Dié-des-Vosges\nUniversité de Lorraine\n2026\n\n##################################################################################################\n\nRéalisé par:\n- Alexandre Brisset (communication VHF, modélisation, fabrication)\n- Matieu Gauthier (modélisation, fabrication)\n- Sasha Guérin--Loison (ensemble de la codebase)\n- Saad Ouadrassi (microcontrôleurs, algorithme de déplacement)\n- Bosco Perrin (conception et fabrication des bateaux)\n- Yasmine ? (conception et fabrication des bateaux)\n\n##################################################################################################\n\nEncadré par:\n- Guillaume Bonfante\n\n##################################################################################################\n\n").yellow();

    println!("{banner_msg}");
}

fn setup_vhosts() -> () {
    let mut child = Command::new("bash")
        .arg("-s")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().expect("Échec ouverture stdin");
    let _ = stdin.write_all(VHOSTS_SETUP_SCRIPT.as_bytes());
    drop(stdin);

    let _ = child.wait();
}

fn are_logfiles_setup() -> bool {
    let config: Config = Config::default();

    for filename in config.log_files_names().iter() {
        if matches!(exists(filename), Ok(false)) {
            return false;
        }
    }

    true
}

fn setup_logfiles() -> () {
    let config: Config = Config::default();

    let _ = config.log_files_names().iter().for_each(|logs_filename| {
        let _ = fs::remove_file(logs_filename);
    });

    let _ = config.log_files_names().iter().for_each(|logs_filename| {
        let path: &Path = Path::new(logs_filename);

        if let Some(parent_dir) = path.parent() {
            fs::create_dir_all(parent_dir);
        }

        File::create(path);
    });
}

fn int_input<T>(prompt: &str, sup: Option<T>) -> T
where
    T: PrimInt + Binary + FromStr + Clone + ToString,
    <T as FromStr>::Err: Display,
{
    let sup: T = sup.unwrap_or(T::max_value());

    Input::<T>::new()
        .with_prompt(format!("\n{}", prompt))
        .validate_with(|val: &T| -> Result<(), &str> {
            if *val <= sup {
                Ok(())
            } else {
                Err("La valeur dépasse le maximum autorisé")
            }
        })
        .interact_text()
        .unwrap()
}

fn ipaddr_input(prompt: &str) -> IpAddr {
    let ip: String = Input::new()
        .with_prompt(format!("\n{}", prompt))
        .validate_with(|val: &String| -> Result<(), &str> {
            if val.parse::<IpAddr>().is_ok() {
                Ok(())
            } else {
                Err("Format d'IPv4 invalide")
            }
        })
        .interact_text()
        .unwrap();

    ip.parse::<IpAddr>().unwrap()
}

fn select_input(prompt: &str, items: &[&str]) -> usize {
    Select::with_theme(&ColorfulTheme::default())
        .default(0)
        .with_prompt(format!("\n{}", prompt))
        .items(items)
        .interact()
        .unwrap()
}

fn tert_input(prompt: &str, pos: Option<&str>, neg: Option<&str>) -> usize {
    let pos: &str = pos.unwrap_or("Oui");
    let neg: &str = neg.unwrap_or("Non");

    let choices: [&str; 3] = [pos, neg, "Retour"];

    select_input(prompt, &choices)
}

fn bool_input(prompt: &str, pos: Option<&str>, neg: Option<&str>) -> bool {
    let pos: &str = pos.unwrap_or("Oui");
    let neg: &str = neg.unwrap_or("Non");

    let choices: [&str; 2] = [pos, neg];

    Select::with_theme(&ColorfulTheme::default())
        .default(1)
        .with_prompt(format!("\n{}", prompt))
        .items(choices)
        .interact()
        .unwrap()
        == 0
}

fn build_config() -> () {
    let mut config: Config = Config::default();

    let is_sim: bool = bool_input(
        "Voulez-vous créer une simulation locale (sur une seule machine) ou mettre en place la maquette réelle ?",
        Some("Simulation (recquiert sudo pour setup les vhosts + simulation à 1 bateau max)"),
        Some("Maquette"),
    );

    if is_sim {
        config.set_is_simulation(true);
        config.set_server_ip(IpAddr::V4(Ipv4Addr::from_str("10.0.0.2").unwrap()));
        config.set_harbourmaster_ip(IpAddr::V4(Ipv4Addr::from_str("10.0.0.1").unwrap()));

        setup_vhosts();
    } else {
        let server_ip: IpAddr = ipaddr_input("Veuillez entrer l'IPv4 du serveur");
        let harbourmaster_ip: IpAddr = ipaddr_input("Veuillez entrer l'IPv4 de la capitainerie");

        config.set_server_ip(server_ip);
        config.set_harbourmaster_ip(harbourmaster_ip);
    }

    config.write();

    println!("\nConfiguration effectuée avec succès !\n")
}

fn change_settings() -> () {
    let config: Config = Config::load().unwrap();

    let settings: [&str; 11] = [
        &format!(
            "Activer / désactiver le mode simulation (valeur actuelle = {}, défaut = Désactivé)",
            if *config.is_simulation() {
                "Activé"
            } else {
                "Désactivé"
            }
        ),
        &format!(
            "Changer l'adresse IP du serveur (valeur actuelle = {}, défaut = 127.0.0.1)",
            config.server_ip().to_string()
        ),
        &format!(
            "Changer l'adresse IP de la capitainerie (valeur actuelle = {}, défaut = 127.0.0.1)",
            config.harbourmaster_ip().to_string()
        ),
        &format!(
            "Activer / désactiver le CLI des logs (valeur actuelle = {}, défaut = Activé)",
            if *config.cli() {
                "Activé"
            } else {
                "Désactivé"
            }
        ),
        &format!(
            "Activer / désactiver l'interface graphique bateau (valeur actuelle = {}, défaut = Activé)",
            if *config.gui() {
                "Activé"
            } else {
                "Désactivé"
            }
        ),
        &format!(
            "Activer / désactiver l'API armateur (valeur actuelle = {}, défaut = Activé)",
            if *config.api() {
                "Activé"
            } else {
                "Désactivé"
            }
        ),
        &format!(
            "Activer / désactiver la détection GPS (valeur actuelle = {}, défaut = Activé)",
            if *config.gps_detection() {
                "Activé"
            } else {
                "Désactivé"
            }
        ),
        &format!(
            "Modifier le délai entre les requêtes GPS bateau (valeur actuelle = {}s, défaut = 5s)",
            *config.gps_refresh_delay()
        ),
        &format!(
            "Modifier le nombres de lignes maximum considérées par le CLI logs (valeur actuelle = {}, défaut = 1000)",
            *config.max_cli_logs_history_length()
        ),
        &format!(
            "Modifier le délai de rafraichissement du CLI logs (valeur actuelle = {}ms, défaut = 100ms)",
            *config.cli_refresh_delay()
        ),
        "Retour",
    ];

    loop {
        let mut config: Config = Config::load().unwrap();

        let param: usize = select_input("Veuillez choisir le paramètre à modifier", &settings);

        match param {
            0 => {
                let choice: usize = tert_input(
                    "Veuillez choisir une option",
                    Some("Activer"),
                    Some("Désactiver"),
                );

                match choice {
                    0 => {
                        config.set_is_simulation(true);
                        setup_vhosts();
                    }
                    1 => {
                        config.set_is_simulation(false);
                    }
                    _ => {}
                }
            }
            1 => {
                let ip: IpAddr = ipaddr_input("Veuillez entrer la nouvelle IPv4 du serveur");

                config.set_server_ip(ip);
            }
            2 => {
                let ip: IpAddr =
                    ipaddr_input("Veuillez entrer la nouvelle IPv4 de la capitainerie");

                config.set_harbourmaster_ip(ip);
            }
            3 => {
                let choice: usize = tert_input(
                    "Veuillez choisir une option",
                    Some("Activer"),
                    Some("Désactiver"),
                );

                match choice {
                    0 => {
                        config.set_cli(true);
                    }
                    1 => {
                        config.set_cli(false);
                    }
                    _ => {}
                }
            }
            4 => {
                let choice: usize = tert_input(
                    "Veuillez choisir une option",
                    Some("Activer"),
                    Some("Désactiver"),
                );

                match choice {
                    0 => {
                        config.set_gui(true);
                    }
                    1 => {
                        config.set_gui(false);
                    }
                    _ => {}
                }
            }
            5 => {
                let choice: usize = tert_input(
                    "Veuillez choisir une option",
                    Some("Activer"),
                    Some("Désactiver"),
                );

                match choice {
                    0 => {
                        config.set_api(true);
                    }
                    1 => {
                        config.set_api(false);
                    }
                    _ => {}
                }
            }
            6 => {
                let choice: usize = tert_input(
                    "Veuillez choisir une option",
                    Some("Activer"),
                    Some("Désactiver"),
                );

                match choice {
                    0 => {
                        config.set_gps_detection(true);
                    }
                    1 => {
                        config.set_gps_detection(false);
                    }
                    _ => {}
                }
            }
            7 => {
                let d: u64 =
                    int_input::<u64>("Veuillez entrer le nouveau délai (en secondes)", None);

                config.set_gps_refresh_delay(d);
            }
            8 => {
                let n: usize =
                    int_input::<usize>("Veuillez entrer le nouveau nombre de lignes maximal", None);

                config.set_max_cli_logs_history_length(n);
            }
            9 => {
                let d: u64 =
                    int_input::<u64>("Veuillez entrer le nouveau délai (en millisecondes)", None);

                config.set_cli_refresh_delay(d);
            }
            _ => {
                break;
            }
        }

        if param <= 9 {
            config.write();

            println!("\nParamètres modifiés avec succès.\n");

            if !are_logfiles_setup() {
                setup_logfiles();
            }
        }
    }
}

fn main() {
    clear_terminal();
    display_banner();

    ctrlc::set_handler(move || {
        clear_terminal();
        display_banner();
    })
    .expect("Erreur lors de la définition du gestionnaire Ctrl+C");

    if sudo::check() == RunningAs::Root {
        println!("Ce simulateur doit être lancé avec sudo.");

        sudo::escalate_if_needed().expect("Erreur lors de l'élévation de privilèges.");
    }

    if Config::load().is_none() {
        println!("Fichier de configuration non trouvé. Lancement du formulaire de création.\n");

        build_config();
    }

    if !are_logfiles_setup() {
        setup_logfiles();
    }

    loop {
        let choice: usize = select_input("Veuillez choisir l'action à effectuer", &FUNCTIONALITIES);

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
                        .expect("\nLe lancement de la capitainerie a échoué\n");

                    clear_terminal();
                    display_banner();
                } else {
                    eprintln!(
                        "\nImpossible de trouver l'exécutable harbourmaster. Demandez-le à Sasha.\n"
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

                        clear_terminal();
                        display_banner();
                    } else {
                        eprintln!(
                            "\nImpossible de trouver l'exécutable server. Demandez-le à Sasha.\n"
                        )
                    }
                } else {
                    eprintln!(
                        "\nLa configuration simulation n'a pas été effectuée ! Veuillez y procéder.\n"
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

                        clear_terminal();
                        display_banner();
                    } else {
                        eprintln!(
                            "\nImpossible de trouver l'exécutable boat. Demandez-le à Sasha.\n"
                        )
                    }
                } else {
                    eprintln!(
                        "\nLa configuration simulation n'a pas été effectuée ! Veuillez y procéder.\n"
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
                        eprintln!(
                            "\nImpossible de trouver l'exécutable server. Demandez-le à Sasha.\n"
                        )
                    }
                } else {
                    eprintln!(
                        "\nLa configuration maquette réelle n'a pas été effectuée ! Veuillez y procéder.\n"
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

                        clear_terminal();
                        display_banner();
                    } else {
                        eprintln!(
                            "\nImpossible de trouver l'exécutable boat. Demandez-le à Sasha.\n"
                        )
                    }
                } else {
                    eprintln!(
                        "\nLa configuration maquette réelle n'a pas été effectuée ! Veuillez y procéder.\n"
                    )
                }
            }
            5 => {
                build_config();
            }
            6 => {
                if bool_input(
                    "En êtes-vous sûr ? Cette action est irréversible",
                    None,
                    None,
                ) {
                    let _ = fs::remove_file("./harbourmaster_database.db");

                    println!("\nDB de la capitainerie supprimée avec succès.\n");
                }
            }
            7 => {
                if bool_input(
                    "En êtes-vous sûr ? Cette action est irréversible",
                    None,
                    None,
                ) {
                    setup_logfiles();

                    println!("\nLogs vidées avec succès.\n");
                }
            }
            8 => {
                change_settings();
            }
            _ => {
                break;
            }
        }
    }
}
