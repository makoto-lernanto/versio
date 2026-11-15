//! La komandlinia enirpunkto de `vs`.

use clap::{Parser, Subcommand};

mod komandoj;

#[derive(Debug, Parser)]
#[command(name = "vs", version, about = "Versiokontrola sistemo")]
struct Argumentoj {
    #[command(subcommand)]
    komando: Komando,
}

#[derive(Debug, Subcommand)]
enum Komando {
    /// Iniciatas novan deponejon en la nuna dosierujo.
    Iniciati,
    /// Registras la nunan staton de la laborspaco kiel novan komiton.
    Komitu {
        /// Mesaĝo de la komito.
        #[arg(short = 'm', long = "mesagxo", value_name = "MESAGXO")]
        mesagxo: String,
    },
}

fn main() -> anyhow::Result<()> {
    let argumentoj = Argumentoj::parse();

    match argumentoj.komando {
        Komando::Iniciati => komandoj::iniciati::plenumi(),
        Komando::Komitu { mesagxo } => komandoj::komitu::plenumi(&mesagxo),
    }
}
