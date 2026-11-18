//! La komandlinia enirpunkto de `vs`.

use clap::{CommandFactory, Parser, Subcommand};

mod komandoj;

/// La helpa ŝablono anstataŭigas la anglajn titolojn de `clap`.
const HELPA_ŜABLONO: &str = "{about-with-newline}\nUzado: {usage}\n\n{all-args}{after-help}";

#[derive(Debug, Parser)]
#[command(
    name = "vs",
    version,
    about = "Versiokontrola sistemo",
    help_template = HELPA_ŜABLONO,
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    subcommand_help_heading = "Komandoj",
    subcommand_value_name = "komando",
    next_help_heading = "Opcioj"
)]
struct Argumentoj {
    #[command(subcommand)]
    komando: Komando,
    /// Montras ĉi tiun helppanon.
    #[arg(
        short = 'h',
        long = "helpo",
        action = clap::ArgAction::Help,
        global = true
    )]
    _helpo: Option<bool>,
    /// Montras la version.
    #[arg(
        short = 'V',
        long = "versio",
        action = clap::ArgAction::Version
    )]
    _versio: Option<bool>,
}

#[derive(Debug, Subcommand)]
enum Komando {
    /// Iniciatas novan deponejon en la nuna dosierujo.
    #[command(help_template = HELPA_ŜABLONO, next_help_heading = "Opcioj")]
    Ini,
    /// Registras la nunan staton de la laborspaco kiel novan komiton.
    #[command(help_template = HELPA_ŜABLONO, next_help_heading = "Opcioj")]
    Komitu {
        /// Noto de la komito.
        #[arg(short = 'm', long = "noto", value_name = "NOTO")]
        noto: String,
    },
    /// Montras la modifitajn, novajn kaj forigitajn dosierojn rilate al KAPO.
    #[command(help_template = HELPA_ŜABLONO, next_help_heading = "Opcioj")]
    Stato,
    /// Montras la komitojn ekde KAPO, la plej nova unue.
    #[command(help_template = HELPA_ŜABLONO, next_help_heading = "Opcioj")]
    Historio {
        /// Kiom da komitoj montri (defaŭlte ĉiujn).
        #[arg(short = 'n', long = "nombro", value_name = "NOMBRO")]
        nombro: Option<usize>,
    },
    /// Restarigas la laborspacon al la arbo de la donita komito.
    #[command(help_template = HELPA_ŜABLONO, next_help_heading = "Opcioj")]
    Eltiri {
        /// Plena aŭ mallonga haketo de la celata komito.
        #[arg(value_name = "HAKETO")]
        haketo: String,
        /// Elfaras la eltiron eĉ kun nekomititaj ŝanĝoj.
        #[arg(short = 'f', long = "perforta")]
        perforta: bool,
    },
    /// Kreas branĉon ĉe KAPO aŭ ĉe la donita komito.
    #[command(help_template = HELPA_ŜABLONO, next_help_heading = "Opcioj")]
    Linio {
        /// Nomo de la nova branĉo.
        #[arg(value_name = "NOMO")]
        nomo: String,
        /// Komito, ĉe kiu krei la branĉon (defaŭlte KAPO).
        #[arg(value_name = "HAKETO")]
        bazo: Option<String>,
    },
    /// Listigas la branĉojn kaj markas la nunan.
    #[command(help_template = HELPA_ŜABLONO, next_help_heading = "Opcioj")]
    Linioj,
    /// Kunfandas branĉon en la nunan branĉon.
    #[command(help_template = HELPA_ŜABLONO, next_help_heading = "Opcioj")]
    Kunfandi {
        /// Nomo de la branĉo, kiun kunfandi.
        #[arg(value_name = "NOMO")]
        linio: String,
    },
    /// Montras la ĝeneralan helppanon.
    #[command(help_template = HELPA_ŜABLONO, next_help_heading = "Opcioj")]
    Helpo,
}

fn main() -> std::process::ExitCode {
    match plenumi() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(eraro) => {
            eprintln!("Eraro: {eraro}");
            let kaŭzoj: Vec<_> = eraro.chain().skip(1).collect();
            if !kaŭzoj.is_empty() {
                eprintln!();
                eprintln!("Kaŭzite de:");
                for kaŭzo in kaŭzoj {
                    eprintln!("    {kaŭzo}");
                }
            }
            std::process::ExitCode::FAILURE
        }
    }
}

fn plenumi() -> anyhow::Result<()> {
    let argumentoj = Argumentoj::parse();

    match argumentoj.komando {
        Komando::Ini => komandoj::iniciati::plenumi(),
        Komando::Komitu { noto } => komandoj::komitu::plenumi(&noto),
        Komando::Stato => komandoj::stato::plenumi(),
        Komando::Historio { nombro } => komandoj::historio::plenumi(nombro),
        Komando::Eltiri { haketo, perforta } => komandoj::eltiri::plenumi(&haketo, perforta),
        Komando::Linio { nomo, bazo } => komandoj::linio::plenumi(&nomo, bazo.as_deref()),
        Komando::Linioj => komandoj::linioj::plenumi(),
        Komando::Kunfandi { linio } => komandoj::kunfandi::plenumi(&linio),
        Komando::Helpo => {
            Argumentoj::command().print_help()?;
            println!();
            Ok(())
        }
    }
}
