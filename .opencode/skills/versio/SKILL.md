---
name: Versio
description: Projekta gvidilo por labori en la deponejo Versio — eta versiokontrola sistemo en Rust kun no_std-kerno, BLAKE3-enhavadresita stokado kaj Kani-pruvoj. Uzu ĉi tion antaŭ ol ŝanĝi kodon, dokumentojn, la DI-laborfluon aŭ la retejon.
---

# Versio — projekta skill

Ĉi tiu skill resumas la regulojn kaj konojn necesajn por sekure labori en la
deponejo Versio. La devigaj fontoj restas `VERSIO.md` kaj `docs/VORTARO.md`;
ĉi tiu dosiero estas rapida gvidilo, ne anstataŭo.

## 1. Identeco de la projekto

- **Versio** (`vs`) estas eta versiokontrola sistemo verkita tute en Rust.
- **Cargo-laborloko** kun tri kestoj:
  - `libversio` — la kernbiblioteko (`#![no_std]` + `alloc`, `#![forbid(unsafe_code)]`).
  - `versio-ki` — la komandlinia ilo `vs` (`std`, `clippy::pedantic`).
  - `no_std_provo` — sendependa `no_std`-sentineleto por DI.
- `arm64_provo/` estas **aparta kesto** (propra `Cargo.lock`, propra lina
  tabelo) nuda-metale konstruata por `aarch64-unknown-none` kaj rulata en QEMU.
- **Eldono de Rust:** 2024, `rust-version = 1.100`.
- **Celo:** venki Git per atako el pli alta dimensio; nuna stato — Fazo 4
  (formala pruvado) finita, Fazo 5 (etikedoj, movhistorio, rubokolekto) en
  evoluo, la resto estas plano. Fidu al `VERSIO.md` kaj al la DI-matrico, ne
  al nombroj el dokumentoj.
- **Licenco:** MIT (`LICENSE`, la sola angla dosiero de la deponejo).

## 2. Lingva regulo

- Ĉio en la deponejo estas **Esperanto**: kodo, identigiloj, komentoj,
  dokumentaro, CLI-eligoj kaj komitmesaĝoj.
- **Komitmesaĝoj tute Esperanto** — titolo, korpo kaj piedlinioj, kiel en
  MiKerno. Angla aŭ ĉina aldono estas nekonforma.
- Restas en la angla nur netuŝeblaj propraj aŭ fiksaj nomoj: `Rust`, celoj
  (`x86_64-unknown-none`…), protokolaj nomoj, `BLAKE3`, `LZ4`, `Kani`,
  `CBMC`, `GitHub Actions`, `QEMU`, `Ed25519`.
- Ne uzu anglajn identigilojn kiel `cpu`, `ram`, `memory`, `thread`,
  `scheduler`, `page`, `frame`, `commit`, `blob`. Uzu la vortaron:
  `procesoro`, `memoro`, `fadeno`, `planisto`, `paĝo`, `kadro`, `komito`,
  `dosiera objekto`.
- Komandnomoj (`vs ini`, `vs stato`, …) uzas nur Askion — sen `ĉ ĝ ĥ ĵ ŝ ŭ`.
  La koncepto estas `branĉo`; la komandoj restas `linio` kaj `linioj`.
- Modulaj dosiernomoj restas Askiaj (ekz. `vojo.rs`); la prozo uzas
  diakritojn. Ne-Askiaj identigiloj kiel `Kerno` estas permesataj.
- `docs/tradukoj/angla.js` estas la angla mapada tavolo de la retejo; ne
  traduku ĝin kaj ne ŝanĝu en ĝi identigilojn, komandnomojn aŭ protokolajn
  nomojn.
- La retejo (`docs/index.html`) kaj ĝia mapado ne devas enkonduki anglajn
  restaĵojn en montratajn tekstojn.

## 3. Versio kaj dokumentoj

- La sola fonto de la versio estas `[workspace.package] version` en
  `Cargo.toml` (nun `0.1.0`). Neniu alia dosiero rajtas porti la version
  sen ĝisdatigo.
- `docs/index.html` portas `data-versio="…"` sur la `<html>`-etikedo; ĝi devas
  kongrui kun `Cargo.toml`.
- Neniu aŭtomata versia ĝisdatigilo ekzistas — ĉe versiaj ŝanĝoj ĝisdatigu
  ambaŭ lokojn permane.

| Dosiero | Rolo |
| --- | --- |
| `README.md` | Gvido kaj rapida starto (Esperanto). |
| `LICENSE` | MIT-licenco (angla, laŭdifine). |
| `VERSIO.md` | Evoluigaj instrukcioj, fazoj kaj DI-reguloj. |
| `docs/VORTARO.md` | Terminologio kaj permesataj anglaj nomoj. |
| `docs/index.html` | La retejo; uzas FluoU-ĵetonojn. |
| `MIKERNO_INTEGRO.md` | Integra pruvo: importi MiKerno-momentfoton. |
| `justfile` | Konstruaj kaj kontrol-celoj. |
| `.github/workflows/ci.yml` | DI: `kontroli-kaj-konstrui` + matrico `kani-kontroli`. |

## 4. Strukturo

```text
libversio/src/
  stokado.rs    # Enhavadresita Stokado (EAS): konservi/ricevi, BLAKE3 + LZ4
  objektoj.rs   # Dosiero, Arbo, Komito; serde + postcard
  deponejo.rs   # Deponejo: .vs/, KAPO, branĉoj, kunfandi, stato, komitu
  indekso.rs    # Laborspaca indekso (INDEKSA_VERSIO = 1)
  kodigo.rs     # Kadra seriigo (Skribilo/Legilo)
  vojo.rs       # Sekura validigo de vojaj komponantoj
  pruvoj.rs     # Ok Kani-pruvoj (nur kun #[cfg(kani)])
  eraroj.rs     # Unueca erartipo
versio-ki/src/
  main.rs       # clap, Esperanta helpa ŝablono (HELPA_ŜABLONO)
  komandoj/     # unu dosiero po subkomando: ini, komitu, stato, historio,
                # eltiri, linio, linioj, kunfandi (+ helpa haketo.rs)
arm64_provo/    # memtesto por bare-metala aarch64 (QEMU)
```

## 5. Konstruo kaj kontrolo

Loka kontrolo (sen Kani):

```sh
cargo fmt --all -- --check
cargo fmt --manifest-path arm64_provo/Cargo.toml -- --check
cargo clippy --workspace --release -- -D warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Nuda-metala memtesto (bezonas `qemu-system-aarch64`):

```sh
just arm64-provo
```

- **`cargo kani` neniam ruliĝas loke.** Kani ruliĝas nur en GitHub Actions.
  Ne rulu `just kani` aŭ `just di` loke.
- **DI** (`.github/workflows/ci.yml`) rulas ĉe ĉiuj puŝoj al `main` kaj ĉiuj
  tirpetoj: unue `Kontroli kaj konstrui` (formo, Clippy, konstruado, la
  `no_std`-sentineleto), poste la matrico `Kani (<pruvo>)`.
- Ok pruvoj: `pruvo_arbo_kanonika`, `pruvo_arbo_duplikatoj`, `pruvo_vojo_sekura`,
  `pruvo_legilo_neniam_panikas`, `pruvo_dekodo_limigita`, `pruvo_komito_seriigo`,
  `pruvo_gepatra_kadro`, `pruvo_kunfanda_kadro`.
- `.github/workflows/ci.yml` estas la sola CI; ne kreu duan kopion.

## 6. DI-reguloj kaj komitoj

- **Kodemanĝoj** devigas DI (neniu `[skip ci]`): puŝu kaj atendu verdan
  matricon. Ne malfortigu pruvojn aŭ lintojn por verdigi ĝin.
- **Nur-dokumentaj ŝanĝoj** (`VERSIO.md`, `MIKERNO_INTEGRO.md`, paĝoj de
  `docs/`, `LICENSE`, `README.md`, `SKILL.md`…) rajtas uzi `[skip ci]`, se
  la komitmesaĝo tion deklaras kaj la deponeja kodo restas netuŝita.
- **Pure nomaj ŝanĝoj** sen konduta efiko rajtas uzi `[skip ci]`, se la
  mesaĝo notas la stabilecon de `fmt`, Clippy kaj testoj.
- **Diru klare**, ĉu DI ruliĝis aŭ estis saltita, kaj ne asertu verdan
  DI antaŭ ol la matrico vere sukcesis.
- Sen klara permeso de la uzanto ne `git commit` aŭ `git push`.
- Komitmesaĝoj en Esperanto; `[skip ci]` estas permesata piedlinio.

## 7. Gravaj interfacaj detaloj

- Objektoj: `Arbo`, `Dosiero`, `Komito`; BLAKE3-256-haketoj sub
  `.vs/objektoj/`; LZ4-kunpremo; postcard-kadroj per `Skribilo`/`Legilo`.
- Dosiersistemo: `DosieraKonservilo` (objektoj), `MemoraKonservilo` (testoj).
- Referencoj: `.vs/referencoj/branĉoj/<nomo>`; la aktiva branĉo donas
  `.vs/branĉo` kaj la `KAPO`-dosiero montras la nunan komitohaketon.
- Indekso: `.vs/indekso`, unua bajto `INDEKSA_VERSIO` (nun `1`).
- Atomikaj skriboj: skribi al provizora dosiero kaj `rename(2)`-i.
- KI-signaloj restas Askiaj: `vs ini`, `vs stato`, `vs historio`,
  `vs komitu --noto`, `vs eltiri`, `vs linio`, `vs linioj`,
  `vs kunfandi`, `vs helpo`, verda Esperanta helpo kontrolata de unuoptesto.
- Ekzistas testo `testo_helpo_estas_tute_esperanta`, kiu rifuzas anglajn
  restaĵojn (`Usage:`, `Options:` …) en la helpa eligo. Ne malaktivigu ĝin.

## 8. Kani-reguloj

- `libversio/src/pruvoj.rs` estas kompilata nur kun `#[cfg(kani)]`; lokaj
  kontroloj ne kaptas erarojn en ĝi — ŝanĝoj al pruvoj validas nur post
  verda DI-matrico.
- Neniam `kani::assume` por mallarĝigi la enigan spacon aŭ kaŝi
  kontraŭekzemplojn; neniam malŝaltu asertojn.
- La plena malkodado de arbitraj `Komito`-kadroj ne estas pruvita (CBMC-limo);
  ĝin kovras unuoptestoj. Ne provu tion promesi al la uzanto.
- La kadraj pruvoj metas fiksajn kampojn antaŭ varilongajn, ĉar simbolaj
  gepatroj tro multekostigus la formulon (77 milionoj da klaŭzoj → 1,4
  milionoj). Ne reordigu nekonsultite.

## 9. Kaptiloj

- **Retaj ekzistantaj nombroj.** README kaj retejo mencias eligon kaj
  nombron de pruvoj; kontrolu `VERSIO.md` kaj la DI-matricon, ne kopiu
  blinde.
- **`data-versio`** en `docs/index.html` kaj la versio en `Cargo.toml`
  devas kongrui; kontrolu ambaŭ ĉe versiaj ŝanĝoj.
- **`no_std`-limo.** `libversio` ne rajtas dependi de `std` (krom per
  `#[cfg(feature = "std")]`); la `std`-realigoj loĝas en `versio-ki` aŭ sub
  trajtoj. `no_std_provo` gardas tion en DI.
- **`#![forbid(unsafe_code)]`** en `libversio`. `arm64_provo` estas la sola
  loko por `unsafe`, kaj ĉiam kun PERMESO-JUSTIFIKO.
- **Askiaj komandnomoj** (`linio`, ne `branĉo create`): la KI restas sen
  diakritoj; la prozo uzas `branĉo`.
- **Unuoptesto `testo_stato_10000_dosieroj_sub_100_ms`** estas ignorata
  ekster la eldona profilo; ne malaktivigu ĝin.
- **Ne enigu `target/` aŭ `.vs/` en git**; ili estas ignorataj.

## 10. Post ĉiu ŝanĝo

1. Rulu la lokajn kontrolojn (vidu §5).
2. Ĝisdatigu `VERSIO.md`, `README.md` kaj `docs/VORTARO.md` se terminologio
   ŝanĝiĝis.
3. Se io en DI ŝanĝiĝas, ruligu DI (puŝo al `main`) — ne `[skip ci]`.
   Se nur dokumentoj ŝanĝiĝis, la komitmesaĝo devas enhavi `[skip ci]` kaj
   deklari, ke la kodo restas netuŝita.
4. Raportu: kion vi ŝanĝis, kion vi rulis, ĉu DI verdis (aŭ estis saltita),
   kaj kiaj limigoj restas.
