use colored::{ColoredString, Colorize};
use dialoguer::{Input, Select};
use shared::config::Config;
use std::{fs, net::IpAddr, process::Command};

fn build_config() -> () {
    let mut config: Config = Config::default();

    let server_ip: String = Input::new()
        .with_prompt("Veuillez entrer l'IPv4 du serveur")
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
        .with_prompt("Veuillez entrer l'IPv4 de la capitainerie")
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

    config.write();

    println!("\nFichier de configuration crée avec succès !\n")
}

fn main() {
    let functionalities: Vec<&str> = vec![
        "Déployer le serveur",
        "Déployer la capitainerie",
        "Déployer un bateau",
        "Relancer la configuration",
        "Quitter",
    ];

    let banner: ColoredString = "
d8888b. db       .d8b.  d888888b d88888b d88888b  .d88b.  d8888b. .88b  d88. d88888b      .88b  d88.  .d8b.  d8888b. d888888b d888888b d888888b .88b  d88. d88888b 
88  `8D 88      d8' `8b `~~88~~' 88'     88'     .8P  Y8. 88  `8D 88'YbdP`88 88'          88'YbdP`88 d8' `8b 88  `8D   `88'   `~~88~~'   `88'   88'YbdP`88 88'     
88oodD' 88      88ooo88    88    88ooooo 88ooo   88    88 88oobY' 88  88  88 88ooooo      88  88  88 88ooo88 88oobY'    88       88       88    88  88  88 88ooooo 
88~~~   88      88~~~88    88    88~~~~~ 88~~~   88    88 88`8b   88  88  88 88~~~~~      88  88  88 88~~~88 88`8b      88       88       88    88  88  88 88~~~~~ 
88      88booo. 88   88    88    88.     88      `8b  d8' 88 `88. 88  88  88 88.          88  88  88 88   88 88 `88.   .88.      88      .88.   88  88  88 88.     
88      Y88888P YP   YP    YP    Y88888P YP       `Y88P'  88   YD YP  YP  YP Y88888P      YP  YP  YP YP   YP 88   YD Y888888P    YP    Y888888P YP  YP  YP Y88888P
".cyan();

    let msg: ColoredString = format!("{banner}\n\n##################################################################################################################################################################\n\nVersion 1.0.0\nEcole Nationale Supérieure des Mines de Nancy, campus ARTEM et de Saint-Dié-des-Vosges, Université de Lorraine, 2026\n\n##################################################################################################################################################################\n\nRéalisé par:\n- Sasha Guérin--Loison (code)\n- Alexandre Brisset (communication VHF, modélisation, fabrication)\n- Saad Ouadrassi (code et algorithme de déplacement)\n- Matieu Gauthier (modélisation, fabrication)\n- Bosco Perrin (conception et fabrication des bateaux)\n- Yasmine ? (conception et fabrication des bateaux)\n\n##################################################################################################################################################################\n\nEncadré par:\n- Guillaume Bonfante\n\n##################################################################################################################################################################\n\n").yellow();

    print!("\x1B[2J\x1B[1;1H");

    println!("{msg}");

    if Config::load().is_none() {
        println!("Fichier de configuration non trouvé. Lancement du formulaire de création.\n");

        build_config();
    }

    while let Ok(choice) = Select::new()
        .with_prompt("Veuillez choisir l'action à effectuer")
        .items(&functionalities)
        .interact()
    {
        match choice {
            0 => {
                if fs::exists("./server").is_ok() {
                    println!("\nLancement du serveur...\n");

                    Command::new("./server")
                        .status()
                        .expect("Le lancement du serveur a échoué");
                } else {
                    println!("Impossible de trouver l'exécutable server. Demandez-le à Sasha.")
                }
            }
            1 => {
                println!("\nLancement de la capitainerie...\n");

                if fs::exists("./server").is_ok() {
                    Command::new("./harbourmaster")
                        .status()
                        .expect("Le lancement de la capitainerie a échoué");
                } else {
                    println!(
                        "Impossible de trouver l'exécutable harbourmaster. Demandez-le à Sasha."
                    )
                }
            }
            2 => {
                println!("\nLancement du bateau...\n");

                if fs::exists("./server").is_ok() {
                    Command::new("./boat")
                        .status()
                        .expect("Le lancement du bateau a échoué");
                } else {
                    println!("Impossible de trouver l'exécutable boat. Demandez-le à Sasha.")
                }
            }
            3 => {
                build_config();
            }
            _ => {
                break;
            }
        }
    }
}
