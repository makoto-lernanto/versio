# Versio (`vs`)

> «La stato, tuj videbla.»

> **Pri la aŭtoreco.** `makoto-lernanto` estas la arkitekto de ĉi tiu projekto:
> li difinas la celojn, la arĥitekturon kaj la pensmanieron. La tuta efektiva
> kodo — same kiel la dokumentaro kaj la testoj — estis verkita de **Qoder**,
> la AI-programada asistanto, kaj ankaŭ kunlabore kun **OpenCode** (modeloj
> **DeepSeek V4.1 Flash Free** kaj **MiMo-V2.6-Flash Free**). La homa
> kontribuo estas la ideo, ne la klavoj.

**Versio** estas eta versiokontrola sistemo verkita en Rust. Ĝi kunigas
enhavadresitan stokadon, rapidan paralelan sencimigo-kalkulon kaj formalan
pruvadon en unu kohera labortablo: `libversio` (`no_std`-kongrueca kerno),
`versio-ki` (la komandlinia ilo `vs`) kaj `arm64_provo` (nuda-metala memtesto).

- **Versio:** 0.1.0 (Cargo-laborloko)
- **Eldono de Rust:** 2024, `rust-version = 1.100`
- **Celoj:** `x86_64` kaj `aarch64` (la kerno ankaŭ konstruiĝas nuda-metale
  por `arm64-unknown-none`)
- **Statuso:** Fazo 4 (formala pruvado) kompletigita, Fazo 5 en evoluo —
  vidu [VERSIO.md](VERSIO.md)
- **Retejo:** [makoto-lernanto.github.io/versio](https://makoto-lernanto.github.io/versio/)
- **Licenco:** [MIT](LICENSE)

---

## Kapabloj

| Kapablo | Stato |
| --- | --- |
| Enhavadresita stokado bazita sur `BLAKE3`-256 kaj `LZ4`-kunpremo | Funkcias |
| `no_std`-kongrueca kerno (`libversio`, `#![no_std]` + `alloc`) | Funkcias |
| Rapido per `rayon`; 10 000-dosiera unuoptesto sub 100 ms en la eldona binaro | Funkcias |
| `vs ini`, `stato`, `komitu`, `historio`, `eltiri` | Funkcias |
| Branĉoj: `vs linio <nomo> [bazo]`, `vs linioj` | Funkcias |
| Tri-voja `kunfandi` kun antaŭenigo kaj konfliktdifekto | Funkcias |
| Detala `KAPO`, branĉoj sub `.vs/referencoj/branĉoj`, laborspaca indekso | Funkcias |
| Atomikaj skriboj per provizora dosiero kaj renomado | Funkcias |
| Ok Kani-pruvoj (arbo-ordo, voja sekureco, legilo, seriigo…) | Funkcias en DI |
| Nuda-metala ARM64-memtesto de la pura kerno (`arm64_provo`) | Funkcias |
| Etikedoj, movhistorio kaj rubokolekto (Fazo 5) | En evoluo |
| `git`-kongrua importi/eksporti (Fazo 9) | Planita |

## Rapida starto

```sh
git clone https://github.com/makoto-lernanto/versio.git
cd versio
cargo build --release
export PATH="$PWD/target/release:$PATH"

mkdir /tmp/versio-provo && cd /tmp/versio-provo
printf 'saluton, mondo\n' > README.md
vs ini
vs stato
vs komitu --noto "Unua versio"
vs historio
vs linio eksperimento
printf 'dua\n' >> README.md
vs komitu --noto "Dua"
vs linioj
vs kunfandi eksperimento
```

La plena helpo estas en Esperanto:

```sh
vs --helpo
vs komitu --helpo
```

## Strukturo de la deponejo

```text
versio/
├── Cargo.toml                 # Cargo-laborloko (versio, lintoj)
├── libversio/                 # Kernbiblioteko (no_std-kongrueca)
│   └── src/                   # stokado.rs, deponejo.rs, objektoj.rs,
│                              # indekso.rs, kodigo.rs, vojo.rs, pruvoj.rs, eraroj.rs
├── versio-ki/                 # La ilo `vs` (std)
│   └── src/komandoj/          # unu dosiero po subkomando
├── no_std_provo/              # Sendependa no_std-sentineleto
├── arm64_provo/               # Nuda-metala ARM64-memtesto (aparta kesto)
├── justfile                   # Konstruo kaj kontrolo
├── .github/workflows/ci.yml   # DI: formo, Clippy, konstruado, Kani-matrico
└── docs/                      # Retejo kaj vortaro
```

## Kontrolado kaj DI

Loka kontrolo (sen Kani):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --release -- -D warnings
cargo test --workspace --all-features
```

Nuda-metala memtesto (bezonas `qemu-system-aarch64`):

```sh
just arm64-provo
```

La formala pruvado rulas nur en GitHub Actions
([`.github/workflows/ci.yml`](.github/workflows/ci.yml)): laboro
`Kontroli kaj konstrui` kaj la matrico `Kani (<pruvo>)` — po unu laboro por
pruvo. Neniam rulu `cargo kani` loke. La deponejo kaj ĝia DI estas verdaj ĉe
la puŝoj al `main`; nur-dokumentaj ŝanĝoj uzas `[skip ci]`.

## Evoluigo

La laborordono de la projekto kaj la akcepto-kriterioj estas en
[VERSIO.md](VERSIO.md). La terminologia vortaro — kiu fiksas ankaŭ la
malpermesitajn anglajn formojn — estas en
[docs/VORTARO.md](docs/VORTARO.md). La reala pruvo pri importado de
MiKerno-momentfoto estas en [MIKERNO_INTEGRO.md](MIKERNO_INTEGRO.md).

Fazo 1–3 konstruis la fundamenton, la datummodelon kaj la staton; Fazo 4 aldonis
la ok formalajn pruvojn kaj la integran provon kun MiKerno. Fazaj 5
(etikedoj, movhistorio, rubokolekto) kaj pli estas en evoluo aŭ planitaj;
vidu la fazajn sekciojn de `VERSIO.md` por la precizaj kriterioj.

## Kontribuado

- Ĉio en la deponejo estas **Esperanto**: kodo, identigiloj, komentoj,
  dokumentaro kaj komitmesaĝoj.
- La komandnomoj restas en Askia formo (sen `ĉ ĝ ĥ ĵ ŝ ŭ`), ekzemple `vs ini`,
  `vs stato`, `vs linioj`.
- Legu unue [VERSIO.md](VERSIO.md) kaj [docs/VORTARO.md](docs/VORTARO.md).
- Ne aldonu anglajn identigilojn aŭ kaŝitajn anglalingvajn aliasojn.
- Kodemanĝo devas ruli DI; nur-dokumentaj ŝanĝoj povas uzi `[skip ci]`.

## Licenco

MIT — vidu [LICENSE](LICENSE).
