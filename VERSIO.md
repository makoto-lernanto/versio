# Evoluigaj instrukcioj por Qoder: la projekto Versio (vs)

> **Rolo**: Vi estas Qoder, respondeca pri verkado de altkvalita, sekura Rust-kodo kongrua kun la Esperantaj nomkonvencioj, laŭ la jenaj striktaj normoj.
> **Celo de la projekto**: konstrui de nulo `versio` (mallongigite `vs`), modernan versiokontrolan sistemon celantan venki Git per atako el pli alta dimensio.
> **Ĉefaj principoj**: sendependa de la deponejo `mikerno`, sed heredanta ĝian inĝenieran animon (Esperanta nomado, rezervita `#![no_std]`-kongrueco, pensmaniero de formala pruvado, minimismo kaj alta efikeco).

---

## 1. Ĉefa arĥitekturo kaj teknikaj normoj

### 1.1 Teknologia stako

- **Lingvo**: Rust (Eldono 2024)
- **Kernbiblioteko (`libversio`)**: `#![no_std]` + `extern crate alloc` (rezervas la kapablon estonte enigi ĝin en la uzantreĝimon de MiKerno).
- **CLI (`versio-cli`)**: norma biblioteko (`std`).
- **Ŝlosilaj dependaĵoj**:
  - `blake3`: enhavadresita haketado (rapidega kaj kolizio-rezistema).
  - `zstd`: objekta kunpremado.
  - `rayon`: paralela dosieruja trairado kaj diff-kalkulado.
  - `clap`: komandlinia analizado (kun la `derive`-trajto ŝaltita).
  - `thiserror` / `anyhow`: erartraktado.
  - `kani`: (enkondukota en Fazo 3) formala pruvado.

### 1.2 Dosierstrukturo

```text
versio/
├── Cargo.toml                 # Radika agordo de la Workspace
├── libversio/                 # Kernbiblioteko (no_std-kongrueca)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs             # Eksportas la publikan API-on
│       ├── stokado.rs         # Enhavadresita stokado (CAS: konservi, ricevi)
│       ├── deponejo.rs        # Deponeja metadatumaro kaj DAG-administrado
│       ├── objektoj.rs        # Datumstrukturoj: Dosiero (Blob), Arbo (Tree), Komito (Commit)
│       └── eraroj.rs          # Unueca erartipo (no_std-kongrueca)
├── versio-cli/                # Komandlinia plenumeblo
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # Enirpunkto
│       └── komandoj/
│           ├── mod.rs
│           ├── iniciati.rs    # vs iniciati (init)
│           ├── stato.rs       # vs stato (status)
│           └── komitu.rs      # vs komitu (commit)
└── .github/workflows/         # CI/CD (cargo check, test, clippy)
```

### 1.3 Nomkonvencioj

- **Kodaj identigiloj**: uzu **Esperanton**. Ekzemple: `Deponejo` (Repository), `Kunfandi` (Merge), `Haketo` (Hash).
- **Komentoj kaj dokumentado**: prefere en Esperanto; por kritikaj, komplikaj logikoj aldonu klarigojn en la ĉina aŭ angla.
- **CLI-komandoj**: Esperantaj verboinfinitivoj aŭ substantivoj (ekz. `iniciati`, `stato`, `historio`).

---

## 2. Fazoj kaj akceptokriterioj

Bonvolu plenumi la fazojn **strikte laŭ ordo**. Post kompletigo de ĉiu fazo, devige atendu la konfirmon de la uzanto antaŭ ol transiri al la sekva fazo.

### 🟢 Fazo 1: Konstruo de la fundamento (La Fundamento)
**Celo**: Starigi la Workspace-on kaj realigi la kernan CAS (enhavadresitan stokadon) sen dependeco de la norma biblioteko.
**Taskoj**:
1. Praŝalti Cargo Workspace-on kaj agordi `libversio` kiel `#![no_std]`.
2. Realigi en `libversio::stokado` la trajton `Konservilo` kaj ĝian realigon bazitan sur la loka dosiersistemo (`DosieraKonservilo`).
3. Realigi `konservi(datumoj: &[u8]) -> Result<Haketo, Eraro>`: kalkulu BLAKE3, kunpremu per zstd kaj konservu en la dosierujon `.vs/objektoj/`.
4. Realigi `ricevi(haketo: &Haketo) -> Result<Vec<u8>, Eraro>`: legu, malkunpremu kaj kontrolu la haketon.

**Akceptokriterioj**:
- [ ] `cargo check --package libversio` sukcesas en medio sen `std`.
- [ ] Verku la unuopteston `testo_konservi_kaj_ricevi`: skribu hazardajn bajtojn; post legado la datumoj devas esti tute identaj kaj la haketo devas kongrui.
- [ ] `cargo test` sukcesas 100%.

### 🟡 Fazo 2: Datummodelo kaj baza CLI (La Datumoj kaj CLI)
**Celo**: Difini la kernajn objektojn de VCS kaj provizi uzeblan komandlinian prototipon.
**Taskoj**:
1. Difini en `libversio::objektoj` la `#[repr(C)]`- aŭ serialigajn strukturojn de `Dosiero` (Blob), `Arbo` (Tree) kaj `Komito` (Commit) (rekomendate uzi `postcard` aŭ `bincode`; konfirmu la `no_std`-subtenon).
2. Realigi `iniciati()` en `libversio::deponejo`, kiu kreu la dosierujan strukturon `.vs`.
3. Integrigi `clap` en `versio-cli` kaj realigi `vs iniciati` kaj `vs komitu --mesagxo "..."`.
4. Logiko de `komitu`: rekursie skani la nunan dosierujon (paralele per `rayon`), preterlasi `.vs`, kalkuli la haketojn de ĉiuj dosieroj, konstrui `Arbo`-n, krei `Komito`-n montrantan al tiu `Arbo`, kaj ĝisdatigi la referencon `HEAD`.

**Akceptokriterioj**:
- [ ] Post rulo de `vs iniciati`, la nuna dosierujo enhavas `.vs/` kun la dosiero `HEAD`.
- [ ] Kreu testan dosieron `testo.txt` kaj rulu `vs komitu --mesagxo "Unua komito"`.
- [ ] Kontrolu, ke sub `.vs/objektoj/` ekzistas kunpremita dosiero kun la responda haketo, kaj ke `HEAD` montras al la haketo de la plej nova `Komito`.

### 🟠 Fazo 3: Trajtoj de atako el pli alta dimensio — Stato kaj momentfotoj (La Superpovo: Stato)
**Celo**: Realigi kontrolon de la laborspaca stato, kiu estas pli rapida kaj pli inteligenta ol `git status`.
**Taskoj**:
1. Realigi `libversio::deponejo::stato()`.
2. Optimumigi la logikon: unue komparu dosierajn metadatumojn (mtime, size) kaj rekalkulu la BLAKE3-haketon nur por ŝanĝitaj dosieroj.
3. Fari paralelan diff (`rayon::par_iter`) inter la laborspaca arbo kaj la `Arbo` montrata de `HEAD`.
4. CLI-eligo: klare listigu la dosierojn `Modifitaj` (Modified), `Novaj` (New) kaj `Forigitaj` (Deleted).

**Akceptokriterioj**:
- [ ] En testa dosierujo kun 10 000 dosieroj, post ŝanĝo de unu dosiero, rulo de `vs stato` devas liveri rezulton en malpli ol 100 ms (danke al la metadatumkaŝmemoro).
- [ ] La eligformato estu pura, sen redunda informo.

### 🔴 Fazo 4: Formala pruvado kaj kunligo kun MiKerno (La Pruvado)
**Celo**: Certigi la absolutan ĝustecon de la kernaj algoritmoj kaj kontroli la kongruecon kun la ideoj de MiKerno.
**Taskoj**:
1. Enkonduki `kani`. Verku Kani-pruvojn por la serialiga/malserialiga logiko en `libversio::objektoj`, aŭ por la haketkontrola logiko en `deponejo`.
2. Verku skripton aŭ dokumenton klarigantan, kiel importi ekzistantan git-deponejon de `mikerno` per `vs` (koncepta pruvo).

**Akceptokriterioj**:
- [ ] `cargo kani` sukcese ruliĝas, pruvante ke la celataj funkcioj havas neniun transpaŝon de limoj kaj neniun nedifinitan konduton.
- [ ] Provizu dokumenton `MIKERNO_INTEGRO.md`, registrantan la preliminajn testrezultojn pri administrado de la fontkodo de `mikerno` per `vs`.

---

## ⚠️ 3. Evitindaj eraroj (GOTCHAS)

Kiel AI, vi devige evitu la jenajn oftajn kaptilojn kiam vi generas kodon:

1. **Poluado de `no_std`**:
   - ❌ Neniam implicite enkonduku `std::`-vojojn en `libversio`.
   - ✅ Devige uzu `core::` kaj `alloc::`. Se dosiersistemaj operacioj estas neevitablaj, abstraktu ilin kiel trajton kaj provizu `std`-realigon en `versio-cli`, aŭ izolu ilin en `libversio` per `#[cfg(feature = "std")]`. *(Noto: en la fazoj 1–2 `libversio` rajtas dependi de `std` por dosiera IO, sed la datumstrukturoj mem devas esti `no_std`-kongruecaj.)*
2. **Blinda fido pri haketaj kolizioj**:
   - ❌ Ne supozu, ke datumoj ne difektiĝos post skribo al disko.
   - ✅ Antaŭ ol redoni datumojn, la funkcio `ricevi` devige rekalkulu la BLAKE3 de la legitaj datumoj kaj komparu ĝin kun la petita `Haketo`; se ili malkongruas, redonu `Eraro::DatumoDifektita`.
3. **Paralelismaj kaptiloj (Rayon)**:
   - ❌ Ne ŝlosu ofte (`Mutex`) en paralelaj iteracioj; tio degradigus la efikecon al seria.
   - ✅ Uzu senŝlosajn datumstrukturojn, aŭ unuece kolektu rezultojn post la paralela kalkulado (ekz. `par_iter().map().collect()`).
4. **Literumaj eraroj en Esperanto**:
   - ❌ Ne miksigu la anglan kaj Esperanton (ekz. `fn get_hash()` aŭ `fn ricevi_hash()`).
   - ✅ Strikte uzu: `Haketo` (Hash), `Deponejo` (Repository), `Dosiero` (File), `Arbo` (Tree), `Komito` (Commit), `Kunfandi` (Merge).
5. **Memorperdo (grandaj dosieroj)**:
   - ❌ Ne legu tutan grandan dosieron samtempe en `Vec<u8>` antaŭ haketado.
   - ✅ Uzu la fluigan API de `blake3::Hasher` (`update`) kune kun bufra legado (ekz. blokoj de 64 KB).

---

## 🔄 4. Laborfluaj instrukcioj por Qoder

Sekvu la jenajn paŝojn ĉiufoje kiam vi respondas al mia peto kaj generas kodon:

1. **Komprenu la intencon**: konfirmu, en kiu fazo (Fazo) vi nuntempe troviĝas.
2. **Kontrolu la limigojn**: revizu la «Evitindajn erarojn» kaj certigu, ke la kodo ne malrespektas la `no_std`-, nomad- aŭ efikec-limigojn.
3. **Generu kodon**: eligi kompletan, kompileblan Rust-kodblokon, inkluzive la necesajn `use`-deklarojn kaj dokumentajn komentojn (`///`).
4. **Provizu testojn**: aldonu `#[cfg(test)]`-unuoptestojn por novaj funkcioj.
5. **Petu konfirmon**: fine de la eligo, klare listigu «Sugestojn por la sekva paŝo» aŭ «Bonvolu ruli la jenajn komandojn por kontroli: `cargo test ...`», kaj atendu mian reagon.

---

## 🤖 Qoder-instrukcio: Konstitucio pri kvalitaj ruĝaj linioj (Kvalita Konstitucio)

> **Valida amplekso**: De ĉi tiu momento, ĉiuj fazoj (inkluzive de la jam finitaj kodoj de Fazo 1/2) devas esti refanditaj por konformi al ĉi tiu konstitucio.
> **La sola difino de finiteco (Definition of Done)**: la laborfluo `CI` en GitHub Actions plene verda (ambaŭ `Lint and build` kaj ĉiuj laboroj `Kani (...)`—vidu la sekcion 5). **Ajna ruĝa lumo = la tasko ne estas finita**; estas malpermesite aserti, ke ĝi estis «jam realigita».

---

### 0. Tri neintertrakteblaj feraj reguloj (Neintertrakteblaj Reguloj)

1. **Malpermesite modifi la kontrolon mem por igi ĝin pasi.** Ne aldonu `#[allow(...)]`, ne malaltigu la nivelon de lint, ne forigu aŭ komentu malsukcesajn testojn aŭ pruvojn, ne uzu `kani::assume` por mallarĝigi la enigan spacon por kaŝi kontraŭekzemplojn.
2. **Se `#[allow]` vere necesas**, devige skribu justigan komenton kaj aparte listigu ĝin en la respondo, atendante la aprobon de la uzanto:
   ```rust
   // PERMESO-JUSTIFIKO: la eligformato de blake3 ĉiam estas 32 bajtoj; la tranĉolongo ĉi tie estas garantiita de la tipo
   #[allow(clippy::indexing_slicing)]
   ```
3. **Ĉe livero devige aldonu realajn eligojn de la komandoj** (la finajn resumojn de `cargo clippy` / `test` / `kani`). Sen eligo = ne finita.

---

## 5. CI/CD: pruvado nur en la nubo (Cloud-only verification)

> **Deviga regulo**: La loka maŝino **neniam** rulu `just kani`, `cargo kani` aŭ la plenan `just ci`. Kodformo, Clippy, konstruado kaj la formala pruvado per Kani plenumiĝas nur en GitHub Actions, por eviti lokan memorelspezon kaj WSL2-malkonekton.

- **Loka laboro**: redakti kodon kaj ruli nur malpezajn kontrolojn, ekzemple `cargo fmt --all -- --check`, `cargo clippy --workspace --release -- -D warnings` kaj `cargo test --workspace --all-features`.
- **Aŭtomata CI**: ĉiu `git push` al `main` kaj ĉiu pull-request rulas `.github/workflows/ci.yml`: unue `Lint and build` (formo, Clippy, konstruado kaj la `no_std`-sentineleto), poste la matrico `Kani (<pruvo>)`.
- **Formala pruvado**: ĉiu pruvo rulas en propra matrica laboro per la oficiala `model-checking/kani-github-action@v1`, kiu instalas Kani kaj rulas `timeout -s KILL 600 cargo kani -p libversio --harness <pruvo>` sur `ubuntu-latest`. `fail-fast: false` certigas, ke ĉiuj pruvoj raportas; 600-sekunda tempolimo kaj 30-minuta laborlimo malebligas, ke unu ŝvebanta pruvo kaŝu la ceterajn aŭ forprenu la protokolojn.
- **Pruvamplekso**: kvin altvaloraj kontroloj — kanonika ordo de arbo, rifuzo de duplikataj vojoj, sekureca validigo de vojoj, neniam-panika legado de nekonfidindaj bajtoj (varintoj kaj limoj; la plena malkodado restas kovrita de unuoptestoj), kaj limigo de deklaritaj kvantoj antaŭ memora rezervado.
- **Kontrolo de rezultoj**: uzu la langeton **Actions** de la deponejo aŭ `gh run list` / `gh run watch`.
- **Difino de finiteco**: la laboro `Lint and build` kaj ĉiuj laboroj `Kani (...)` devas esti verdaj en GitHub Actions.

