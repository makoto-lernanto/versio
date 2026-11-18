//! La komandlinia enirpunkto de `vs`.

use clap::{CommandFactory, Parser, Subcommand};

mod komandoj;

/// La helpa ŝablono anstataŭigas la anglajn titolojn de `clap`.
const HELPA_SXABLONO: &str = "{about-with-newline}\nUzado: {usage}\n\n{all-args}{after-help}";

#[derive(Debug, Parser)]
#[command(
    name = "vs",
    version,
    about = "Versiokontrola sistemo",
    help_template = HELPA_SXABLONO,
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
        alias = "help",
        action = clap::ArgAction::Help,
        global = true
    )]
    _helpo: Option<bool>,
    /// Montras la version.
    #[arg(
        short = 'V',
        long = "versio",
        alias = "version",
        action = clap::ArgAction::Version
    )]
    _versio: Option<bool>,
}

#[derive(Debug, Subcommand)]
enum Komando {
    /// Iniciatas novan deponejon en la nuna dosierujo.
    #[command(help_template = HELPA_SXABLONO, next_help_heading = "Opcioj")]
    Ini,
    /// Registras la nunan staton de la laborspaco kiel novan komiton.
    #[command(help_template = HELPA_SXABLONO, next_help_heading = "Opcioj")]
    Komitu {
        /// Noto de la komito.
        #[arg(short = 'm', long = "noto", value_name = "NOTO")]
        noto: String,
    },
    /// Montras la modifitajn, novajn kaj forigitajn dosierojn rilate al HEAD.
    #[command(help_template = HELPA_SXABLONO, next_help_heading = "Opcioj")]
    Stato,
    /// Montras la komitojn ekde HEAD, la plej nova unue.
    #[command(help_template = HELPA_SXABLONO, next_help_heading = "Opcioj")]
    Historio {
        /// Kiom da komitoj montri (defaŭlte ĉiujn).
        #[arg(short = 'n', long = "nombro", value_name = "NOMBRO")]
        nombro: Option<usize>,
    },
    /// Restarigas la laborspacon al la arbo de la donita komito.
    #[command(help_template = HELPA_SXABLONO, next_help_heading = "Opcioj")]
    Eltiri {
        /// Plena aŭ mallonga haketo de la celata komito.
        #[arg(value_name = "HAKETO")]
        haketo: String,
        /// Elfaras la eltiron eĉ kun nekomititaj ŝanĝoj.
        #[arg(short = 'f', long = "perforta")]
        perforta: bool,
    },
    /// Kreas branĉon ĉe HEAD aŭ ĉe la donita komito.
    #[command(help_template = HELPA_SXABLONO, next_help_heading = "Opcioj")]
    Brancxo {
        /// Nomo de la nova branĉo.
        #[arg(value_name = "NOMO")]
        nomo: String,
        /// Komito, ĉe kiu krei la branĉon (defaŭlte HEAD).
        #[arg(value_name = "HAKETO")]
        bazo: Option<String>,
    },
    /// Listigas la branĉojn kaj markas la nunan.
    #[command(help_template = HELPA_SXABLONO, next_help_heading = "Opcioj")]
    Brancxoj,
    /// Montras la ĝeneralan helppanon.
    #[command(help_template = HELPA_SXABLONO, next_help_heading = "Opcioj")]
    Helpo,
}

fn main() -> anyhow::Result<()> {
    let argumentoj = Argumentoj::parse();

    match argumentoj.komando {
        Komando::Ini => komandoj::iniciati::plenumi(),
        Komando::Komitu { noto } => komandoj::komitu::plenumi(&noto),
        Komando::Stato => komandoj::stato::plenumi(),
        Komando::Historio { nombro } => komandoj::historio::plenumi(nombro),
        Komando::Eltiri { haketo, perforta } => komandoj::eltiri::plenumi(&haketo, perforta),
        Komando::Brancxo { nomo, bazo } => komandoj::brancxo::plenumi(&nomo, bazo.as_deref()),
        Komando::Brancxoj => komandoj::brancxoj::plenumi(),
        Komando::Helpo => {
            Argumentoj::command().print_help()?;
            println!();
            Ok(())
        }
    }
}
