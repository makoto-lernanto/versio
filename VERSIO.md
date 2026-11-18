# Evoluigaj instrukcioj por Qoder: la projekto Versio (vs)

> **Rolo**: Vi estas Qoder, respondeca pri verkado de altkvalita, sekura Rust-kodo kongrua kun la Esperantaj nomkonvencioj, laŭ la jenaj striktaj normoj.
> **Celo de la projekto**: konstrui de nulo `versio` (mallongigite `vs`), modernan versiokontrolan sistemon celantan venki Git per atako el pli alta dimensio.
> **Ĉefaj principoj**: sendependa de la deponejo `mikerno`, sed heredanta ĝian inĝenieran animon (Esperanta nomado, rezervita `#![no_std]`-kongrueco, pensmaniero de formala pruvado, minimismo kaj alta efikeco).

---

## 1. Ĉefa arĥitekturo kaj teknikaj normoj

### 1.1 Teknologia stako

- **Lingvo**: Rust (Eldono 2024)
- **Kernbiblioteko (`libversio`)**: `#![no_std]` + `extern crate alloc` (rezervas la kapablon estonte enigi ĝin en la uzantreĝimon de MiKerno).
- **KI (Komandliniaj Iloj) (`versio-ki`)**: norma biblioteko (`std`).
- **Ŝlosilaj dependaĵoj**:
  - `blake3`: enhavadresita haketado (rapidega kaj kolizio-rezistema).
  - `lz4_flex`: objekta kunpremado.
  - `rayon`: paralela dosieruja trairado kaj kompara kalkulado.
  - `clap`: komandlinia analizado (kun la `derive`-trajto ŝaltita).
  - `thiserror` / `anyhow`: erartraktado.
  - `kani`: formala pruvado (enkondukita en Fazo 4, rulata nur en DI — Daŭra Integrado).

### 1.2 Dosierstrukturo

```text
versio/
├── Cargo.toml                 # Radika agordo de la Cargo-laborloko
├── libversio/                 # Kernbiblioteko (no_std-kongrueca)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs             # Eksportas la publikan interfacon
│       ├── stokado.rs         # Enhavadresita stokado (EAS: Enhavadresita Stokado)
│       ├── deponejo.rs        # Deponeja metadatumaro, grafa administrado kaj kunfandado
│       ├── objektoj.rs        # Datumstrukturoj: Dosiero, Arbo, Komito
│       ├── indekso.rs         # Metadatumkaŝmemoro por `stato` kaj `komitu`
│       ├── kodigo.rs          # Legado kaj skribado de la postcard-formato
│       ├── vojo.rs            # Validigo de sekuraj vojaj komponantoj
│       ├── pruvoj.rs          # Kani-pruvoj (nur kun `cfg(kani)`)
│       └── eraroj.rs          # Unueca erartipo (no_std-kongrueca)
├── no_std_provo/              # Sendependa no_std-sentineleto por DI
├── versio-ki/                 # KI: komandliniaj iloj
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # Enirpunkto
│       └── komandoj/
│           ├── mod.rs
│           ├── iniciati.rs    # `vs ini` — krei deponejon
│           ├── stato.rs       # `vs stato` — montri la ŝanĝojn
│           ├── historio.rs    # `vs historio` — listigi komitojn
│           ├── eltiri.rs      # `vs eltiri` — restarigi la arbon
│           ├── linio.rs       # `vs linio` — krei branĉon
│           ├── linioj.rs      # `vs linioj` — listigi branĉojn
│           ├── kunfandi.rs    # `vs kunfandi` — kunfandi branĉon
│           ├── haketo.rs      # komuna haket-solvado
│           └── komitu.rs      # `vs komitu` — registri komiton
└── .github/workflows/         # DI/DL (kontrolo, provoj, Clippy)
```

### 1.3 Nomkonvencioj

- **Kodaj identigiloj**: uzu **Esperanton**. Ekzemple: `Deponejo`, `Kunfandi`, `Haketo`.
- **Komentoj kaj dokumentado**: nur en Esperanto.
- **KI-komandoj**: Esperantaj nomoj sen anglaj terminoj; la komenca komando estas la mallongigo `ini`, la ceteraj estas plenaj vortoj (ekz. `stato`, `historio`, `kunfandi`). Komandnomoj kaj argumentaj nomoj uzas nur ASKION (sen `ĉ ĝ ĥ ĵ ŝ ŭ`), por eviti terminalajn kodigajn problemojn. La koncepto estas `branĉo`, sed la komandnomoj restas `linio` kaj `linioj` pro la ASKIA limo.

---

## 2. Fazoj kaj akceptokriterioj

Bonvolu plenumi la fazojn **strikte laŭ ordo**. Post kompletigo de ĉiu fazo, devige atendu la konfirmon de la uzanto antaŭ ol transiri al la sekva fazo.

### 🟢 Fazo 1: Konstruo de la fundamento (La Fundamento)
**Celo**: Starigi la Cargo-laborlokon kaj realigi la kernan EAS (Enhavadresitan Stokadon) sen dependeco de la norma biblioteko.
**Taskoj**:
1. Praŝalti Cargo-laborlokon kaj agordi `libversio` kiel `#![no_std]`.
2. Realigi en `libversio::stokado` la trajton `Konservilo` kaj ĝian realigon bazitan sur la loka dosiersistemo (`DosieraKonservilo`).
3. Realigi `konservi(datumoj: &[u8]) -> Result<Haketo, Eraro>`: kalkulu BLAKE3, kunpremu per `lz4_flex` kaj konservu en la dosierujon `.vs/objektoj/`.
4. Realigi `ricevi(haketo: &Haketo) -> Result<Vec<u8>, Eraro>`: legu, malkunpremu kaj kontrolu la haketon.

**Akceptokriterioj**:
- [ ] `cargo check --package libversio` sukcesas en medio sen `std`.
- [ ] Verku la unuopteston `testo_konservi_kaj_ricevi`: skribu hazardajn bajtojn; post legado la datumoj devas esti tute identaj kaj la haketo devas kongrui.
- [ ] `cargo test` sukcesas 100%.

### 🟡 Fazo 2: Datummodelo kaj baza KI (La Datumoj kaj KI)
**Celo**: Difini la kernajn objektojn de la versiokontrola sistemo kaj provizi uzeblan komandlinian prototipon.
**Taskoj**:
1. Difini en `libversio::objektoj` la `#[repr(C)]`- aŭ serialigajn strukturojn de `Dosiero`, `Arbo` kaj `Komito` (rekomendate uzi `postcard` aŭ `bincode`; konfirmu la `no_std`-subtenon).
2. Realigi `iniciati()` en `libversio::deponejo`, kiu kreu la dosierujan strukturon `.vs`.
3. Integrigi `clap` en `versio-ki` kaj realigi `vs ini` kaj `vs komitu --noto "..."`.
4. Logiko de `komitu`: paralele skani la nunan dosierujon per `rayon`, preterlasi `.vs`, kalkuli aŭ reuzi (per la indekso) la haketojn de la dosieroj, konstrui `Arbo`-n, krei `Komito`-n montrantan al tiu `Arbo`, kaj ĝisdatigi la referencon `KAPO`.

**Akceptokriterioj**:
- [ ] Post rulo de `vs ini`, la nuna dosierujo enhavas `.vs/` kun la dosiero `KAPO`.
- [ ] Kreu testan dosieron `testo.txt` kaj rulu `vs komitu --noto "Unua komito"`.
- [ ] Kontrolu, ke sub `.vs/objektoj/` ekzistas kunpremita dosiero kun la responda haketo, kaj ke `KAPO` montras al la haketo de la plej nova `Komito`.

### 🟠 Fazo 3: Trajtoj de atako el pli alta dimensio — Stato kaj momentfotoj (La Superpovo: Stato)
**Celo**: Realigi kontrolon de la laborspaca stato, kiu estas pli rapida kaj pli inteligenta ol `git status`.
**Taskoj**:
1. Realigi `libversio::deponejo::stato()`.
2. Optimumigi la logikon: unue komparu dosierajn metadatumojn (modiftempo, grando) kaj rekalkulu la BLAKE3-haketon nur por ŝanĝitaj dosieroj.
3. Fari paralelan komparadon (`rayon::par_iter`) inter la laborspaca arbo kaj la `Arbo` montrata de `KAPO`.
4. KI-eligo: klare listigu la dosierojn `Modifitaj`, `Novaj` kaj `Forigitaj`.

**Akceptokriterioj**:
- [ ] En testa dosierujo kun 10 000 dosieroj, post ŝanĝo de unu dosiero, rulo de `vs stato` devas liveri rezulton en malpli ol 100 ms (danke al la metadatumkaŝmemoro).
- [ ] La eligformato estu pura, sen redunda informo.

### 🔴 Fazo 4: Formala pruvado kaj kunligo kun MiKerno (La Pruvado)
**Celo**: Certigi la absolutan ĝustecon de la kernaj algoritmoj kaj kontroli la kongruecon kun la ideoj de MiKerno.
**Taskoj**:
1. Enkonduki `kani`. Verku Kani-pruvojn por la serialiga/malserialiga logiko en `libversio::objektoj`, aŭ por la haketkontrola logiko en `deponejo`.
2. Verku skripton aŭ dokumenton klarigantan, kiel importi ekzistantan git-deponejon de `mikerno` per `vs` (koncepta pruvo).

**Akceptokriterioj**:
- [x] `cargo kani` sukcese ruliĝas, pruvante ke la celataj funkcioj havas neniun transpaŝon de limoj kaj neniun nedifinitan konduton.
- [x] Provizu dokumenton `MIKERNO_INTEGRO.md`, registrantan la preliminajn testrezultojn pri administrado de la fontkodo de `mikerno` per `vs`.

La jenaj fazoj estas plano, ne promeso; la ordo kaj la amplekso povas ŝanĝiĝi laŭ mezuritaj bezonoj. Ĉiu fazo postulas verdan DI antaŭ ol la sekva komenciĝas, kaj neniu fazo rajtas malfortigi la antaŭajn kontrolojn.

### Fazo 5: Historio kaj branĉoj kiel unuaklasaj objektoj
**Celo**: Firmigi la jam ekzistantajn komandojn `linio`, `linioj`, `historio`, `eltiri` kaj `kunfandi` per kompleta referenca modelo: etikedoj, movhistorio, detaĉita `KAPO`, forigo kaj renomado de branĉoj, kaj sekura rubokolekto.

**Taskoj**:
1. Difini `Etikedo` kaj `Movhistorio` en `libversio::deponejo`; konservi ilin sub `.vs/referencoj/etikedoj` kaj `.vs/movhistorio`.
2. Aldoni la komandojn `vs etikedi`, `vs etikedoj`, `vs renomi`, `vs forigi` (por branĉo) kaj `vs movhistorio`.
3. Registri ĉiun movon de `KAPO` kaj de branĉo en la movhistorion, inkluzive de kunfandoj kaj antaŭenigoj.
4. Realigi `vs rubokolekti`: forigi nur objektojn neatingeblajn el `KAPO`, branĉoj, etikedoj kaj movhistorio.

**Akceptokriterioj**:
- [ ] `vs etikedi v1.0` kaj `vs historio v1.0` funkcias; etikedo povas montri al komito aŭ al alia etikedo.
- [ ] `vs movhistorio` montras la antaŭajn valorojn de `KAPO` kaj de ĉiu branĉo.
- [ ] `vs rubokolekti` forigas nur neatingeblajn objektojn; unuoptesto pruvas, ke ĉiuj atingeblaj restas.
- [ ] La atingo-kalkulo havas Kani-pruvon aŭ estas kovrita de la ekzistanta pruvo pri la legilo; la DI-matrico restas verda.

### Fazo 6: Reta kunlaboro sen centra servilo
**Celo**: Ebligi interŝanĝon inter deponejoj per simpla, rekomencigebla kaj aŭtentikigita protokolo, sen postuli specialan servilon.

**Taskoj**:
1. Difini malproksimajn deponejojn en `.vs/malproksimoj`; aldonu `vs malproksima` por administri ilin.
2. Realigi `vs publikigi` kaj `vs akiri`: transdoni nur la objektojn, kiujn la alia flanko ankoraŭ ne havas.
3. Subteni rekomencon post interrompo kaj kontroli ĉiun ricevitan objekton per ĝia haketo.
4. Aŭtentikigi per SSH-ŝlosiloj aŭ subskribitaj ĵetonoj; neniam konservi pasvortojn en `.vs/`.
5. Dokumenti la protokolajn mesaĝojn kaj iliajn limojn en aparta `PROTOKOLO.md`.

**Akceptokriterioj**:
- [ ] Du lokaj deponejoj interŝanĝas komitojn kaj branĉojn tra loka HTTP-servilo per `vs publikigi` kaj `vs akiri`.
- [ ] Interrompita transdono rekomenciĝas sen rekopii la jam konatajn objektojn.
- [ ] Difektita objekto haltigas la transdonon antaŭ ol ĝi eniras la stokejon.
- [ ] Neniu sekreto aperas en `.vs/`, en la protokolo aŭ en la protokoloj de DI.

### Fazo 7: Grandskala efikeco
**Celo**: Superi `git status` kaj `git add`/`git commit` ĉe 100 000 dosieroj, grandaj dosieroj kaj malrapidaj diskaj subteniloj.

**Taskoj**:
1. Evoluigi la indekson al binara, mempriskriba formato kun versio, atomaj ĝisdatigoj kaj difekta kontrolo.
2. Paraleligi haketadon kaj komparadon per `rayon`, sed limigi la memoruzon per fluado.
3. Aldoni inkrementan arbokonstruadon: reuzi subarbojn, kies enhavo ne ŝanĝiĝis.
4. Verki benchmarkojn por `stato`, `komitu`, `eltiri` kaj `historio`; la benchmarkoj ne rulas en la ĉefa DI.

**Akceptokriterioj**:
- [ ] `vs stato` finiĝas en malpli ol 100 ms ĉe 100 000 dosieroj post unu ŝanĝo (release, varma kaŝmemoro).
- [ ] `vs komitu` uzas malpli ol 256 MiB da memoro ĉe 1 GiB da dosieroj.
- [ ] Reuzo de subarboj estas pruvita per unuoptesto kaj mezurita per benchmarko.
- [ ] Neniu ekzistanta testo aŭ pruvo regresas; la DI-matrico restas verda.

### Fazo 8: Sekureco, subskriboj kaj daŭripova stokado
**Celo**: Garantii la aŭtentecon kaj integrecon de komitoj, arboj kaj dosieraj objektoj.

**Taskoj**:
1. Subskribi komitojn per Ed25519; aldonu `vs subskribi` kaj `vs kontroli`.
2. Konservi publikajn ŝlosilojn en `.vs/ŝlosiloj` kaj politikojn en `.vs/agordo`.
3. Kontroli subskribojn dum `eltiri` kaj `kunfandi`; permesi politikon `postuli-subskribon`.
4. Certigi, ke `vs rubokolekti` ne forigas subskribitajn objektojn, kiujn la politiko protektas.
5. Aldoni fuzz-testojn por la malkodado kaj daŭrigi la Kani-pruvojn pri la legilo.

**Akceptokriterioj**:
- [ ] Ŝanĝita komito aŭ arbo estas rifuzata, se ĝia subskribo ne validas.
- [ ] Subskribo de unu komito validas tra `kunfandi` kaj `eltiri`.
- [ ] Fuzz-kampanjo de 24 horoj ne trovas panikon en la malkodado.
- [ ] La Kani-matrico restas verda kaj neniu `#[allow]` estas aldonita.

### Fazo 9: Kongrueco kun Git kaj migrado
**Celo**: Povi legi kaj skribi Git-deponejojn sen perdi historion, branĉojn aŭ etikedojn.

**Taskoj**:
1. Legi Git-pack-dosierojn, `refs` kaj la indekson; mapi Git-objektojn al la Versio-datummodelo.
2. Skribi Git-kongruan arbon kaj komiton, kiujn `git log` kaj `git fsck` akceptas.
3. Realigi `vs importi` kaj `vs eksporti` por komitoj, branĉoj kaj etikedoj; dokumenti la limojn ĉe submoduloj kaj simbolaj ligiloj.
4. Raporti malsamajn kodigojn de dosiernomoj anstataŭ silente konverti ilin.

**Akceptokriterioj**:
- [ ] `vs importi <git-deponejo>` konservas ĉiun komiton, branĉon kaj etikedon de la testa MiKerno-deponejo.
- [ ] `git fsck` akceptas la eksportitan deponejon.
- [ ] Rondiro Versio→Git→Versio konservas la arbo-identigon kaj la komitajn mesaĝojn.
- [ ] Ĉiuj limoj aperas en la dokumentaro kaj en la testoj.

### Fazo 10: Integriĝo kun MiKerno
**Celo**: Uzi la saman kernon en la rulanta MiKerno, sen `std` kaj sen hosta dosiersistemo.

**Taskoj**:
1. Certigi, ke `libversio` restas `no_std` kaj daŭre trapasas la sentinelon en DI.
2. Provizi adaptilon por la MiKerno-dosiersistemo (SNDS aŭ posta skriba dosiersistemo) kaj por la kapabla modelo.
3. Enkonduki komandon en Horizonto por krei kaj restarigi momentfoton de la laborspaco.
4. Dokumenti la limojn: neniuj simbolaj ligiloj, permesbitoj aŭ sekurigitaj dosieroj.

**Akceptokriterioj**:
- [ ] `libversio` konstruiĝas sen `std` kaj la sentinelo restas verda.
- [ ] Horizonto kreas kaj restarigas momentfoton en QEMU.
- [ ] `MIKERNO_INTEGRO.md` estas ĝisdatigita per la novaj limoj kaj la mezuritaj rezultoj.
- [ ] Neniu kerna dependeco al `std` aŭ al hosta dosiersistemo.

### Fazo 11: Stabileco kaj eldono 1.0
**Celo**: Fiksi la datumformatojn, dokumenti la migradon kaj publikigi stabilan eldonon.

**Taskoj**:
1. Versii ĉiun surdiskan formaton kaj skribi migradojn el la antaŭaj versioj.
2. Aldoni kongruectestojn inter eldonoj kaj testi ĝisdatigon de malnova deponejo.
3. Prepari pakaĵojn, manlibrajn paĝojn kaj ripeteblajn konstruaĵojn.
4. Publikigi `1.0` kun subskribitaj aktivaĵoj kaj klara listo de limoj.

**Akceptokriterioj**:
- [ ] Deponejo kreita per la antaŭa eldono malfermiĝas kaj funkcias sen permana riparo.
- [ ] La datumformatoj havas version kaj migradon kovritan de testoj.
- [ ] `cargo install` kaj la elŝutitaj pakaĵoj produktas funkciantan `vs`.
- [ ] La eldono `1.0` havas verdan DI-matricon kaj subskribitajn aktivaĵojn.

---

## ⚠️ 3. Evitindaj eraroj (Kaptiloj)

Kiel AI, vi devige evitu la jenajn oftajn kaptilojn kiam vi generas kodon:

1. **Poluado de `no_std`**:
   - ❌ Neniam implicite enkonduku `std::`-vojojn en `libversio`.
   - ✅ Devige uzu `core::` kaj `alloc::`. Se dosiersistemaj operacioj estas neevitablaj, abstraktu ilin kiel trajton kaj provizu `std`-realigon en `versio-ki`, aŭ izolu ilin en `libversio` per `#[cfg(feature = "std")]`. *(Noto: en la fazoj 1–2 `libversio` rajtas dependi de `std` por dosiera IO, sed la datumstrukturoj mem devas esti `no_std`-kongruecaj.)*
2. **Blinda fido pri haketaj kolizioj**:
   - ❌ Ne supozu, ke datumoj ne difektiĝos post skribo al disko.
   - ✅ Antaŭ ol redoni datumojn, la funkcio `ricevi` devige rekalkulu la BLAKE3 de la legitaj datumoj kaj komparu ĝin kun la petita `Haketo`; se ili malkongruas, redonu `Eraro::DatumoDifektita`.
3. **Paralelismaj kaptiloj (Rayon)**:
   - ❌ Ne ŝlosu ofte (`Mutex`) en paralelaj iteracioj; tio degradigus la efikecon al seria.
   - ✅ Uzu senŝlosajn datumstrukturojn, aŭ unuece kolektu rezultojn post la paralela kalkulado (ekz. `par_iter().map().collect()`).
4. **Literumaj eraroj en Esperanto**:
   - ❌ Ne miksigu la anglan kaj Esperanton (ekz. `fn get_hash()` aŭ `fn ricevi_hash()`).
   - ✅ Strikte uzu: `Haketo`, `Deponejo`, `Dosiero`, `Arbo`, `Komito`, `Kunfandi`.
5. **Memorperdo (grandaj dosieroj)**:
   - ❌ Ne legu tutan grandan dosieron samtempe en `Vec<u8>` antaŭ haketado.
   - ✅ Uzu la fluigan interfacon de `blake3::Hasher` (`update`) kune kun bufra legado (ekz. blokoj de 64 KB).

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
> **La sola difino de finiteco**: la laborfluo `DI` en GitHub Actions plene verda (ambaŭ `Kontroli kaj konstrui` kaj ĉiuj laboroj `Kani (...)`—vidu la sekcion 5). **Ajna ruĝa lumo = la tasko ne estas finita**; estas malpermesite aserti, ke ĝi estis «jam realigita».

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

## 5. DI/DL (Daŭra Integrado / Daŭra Liverado): pruvado nur en la nubo

> **Deviga regulo**: La loka maŝino **neniam** rulu `just kani`, `cargo kani` aŭ la plenan `just di`. Kodformo, Clippy, konstruado kaj la formala pruvado per Kani plenumiĝas nur en GitHub Actions, por eviti lokan memorelspezon kaj WSL2-malkonekton.

- **Loka laboro**: redakti kodon kaj ruli nur malpezajn kontrolojn, ekzemple `cargo fmt --all -- --check`, `cargo clippy --workspace --release -- -D warnings` kaj `cargo test --workspace --all-features`.
- **Aŭtomata DI**: ĉiu `git push` al `main` kaj ĉiu tirpeto rulas `.github/workflows/di.yml`: unue `Kontroli kaj konstrui` (formo, Clippy, konstruado kaj la `no_std`-sentineleto), poste la matrico `Kani (<pruvo>)`.
- **Formala pruvado**: ĉiu pruvo rulas en propra matrica laboro per la oficiala `model-checking/kani-github-action@v1`, kiu instalas Kani kaj rulas `timeout -s KILL 600 cargo kani -p libversio --harness <pruvo>` sur `ubuntu-latest`. `fail-fast: false` certigas, ke ĉiuj pruvoj raportas; 600-sekunda tempolimo kaj 30-minuta laborlimo malebligas, ke unu ŝvebanta pruvo kaŝu la ceterajn aŭ forprenu la protokolojn.
- **Pruvamplekso**: ok kontroloj — la kvin ekzistantaj pruvoj pri kanonika arbo-ordo, duplikataj vojoj, voja sekureco, legado de nekonfidindaj bajtoj kaj limigo de deklaritaj kvantoj; plus tri komitaj seriigaj pruvoj per la eksplicita `Skribilo` — radika sen-gepatra kadro (36–46 bajtoj), ordinara unu-gepatra kadro (68–78 bajtoj) kaj kunfanda kadro kun du gepatroj (100–110 bajtoj). Ĉiu kadra pruvo havas simbolikan radikan haketon, simbolikan ĉeeston de mesaĝo kaj ajnan `u64`-tempstampon; la gepatroj estas fiksaj, ĉar neniu aserto komparas iliajn bajtojn. La tri kadraj formoj havas proprajn pruvojn, ĉar simbolaj gepatroj unuigus ĉiujn formojn en unu tro multekosta formulo. La kadro metas la fiksajn kampojn (haketoj) antaŭ la varilongajn (mesaĝo, tempstampo), do la 32-bajtaj kopioj okazas ĉe konstantaj pozicioj; la formulo de la kunfanda pruvo tiel restas ĉirkaŭ 1,4 milionoj da klaŭzoj anstataŭ 77 milionoj. Plena malkodado de arbitraj `Komito`-kadroj ne estas pruvita per Kani: la UTF-8- kaj arbo-malkodaj vojoj superas la mezuritan formulbuĝeton de la DI eĉ kun konkretaj kadroj. Tiun kampon kovras unuoptestoj (rondiroj, etikedoj, tranĉoj, varintaj limoj) kaj la arbitraj-bajta pruvo pri la legilo. Vidu `MIKERNO_INTEGRO.md` por la preciza amplekso, la mezuritaj limoj, realaj integratestoj kaj la nuba stato.
- **Kani-kodo ne kompiliĝas loke**: `libversio/src/pruvoj.rs` estas `#[cfg(kani)]`, do lokaj `fmt`, `Clippy` kaj testoj ne kaptas erarojn en ĝi. Ĉiu ŝanĝo al la pruvoj validas nur post verda DI-matrico.
- **Nuba stato**: revizio `580dfb8` sukcese finis la DI-rulon [`44302583687`](https://github.com/makoto-lernanto/versio/actions/runs/44302583687) — `Kontroli kaj konstrui` kaj ĉiuj ok `Kani (...)`-laboroj verdaj (2026-11-18).
- **Aŭtomata liverado**: post kodŝanĝoj kaj la permesitaj lokaj kontroloj, aŭtomate kreu novan komiton kaj puŝu ĝin por ekigi DI, sen aparta konfirmo. Se DI malsukcesas, analizu la protokolojn, korektu la kaŭzon kaj denove komitu kaj puŝu; ne malfortigu kontrolojn. Tiu permeso ne inkluzivas devigan puŝon aŭ reverkadon de historio.
- **Nur-dokumentaj ŝanĝoj**: se nur dokumentoj (ekz. `VERSIO.md`, `MIKERNO_INTEGRO.md`) ŝanĝiĝas kaj la ĉefa kodo, testoj aŭ DI-agordo restas netuŝitaj, metu `[skip ci]` en la komitan mesaĝon kaj klarigu, ke la ĉefa kodo ne ŝanĝiĝis; tiam DI ne bezonas ruli.
- **Pure nomaj ŝanĝoj**: terminologiaj aŭ nomaj ŝanĝoj sen efiko al la konduto (ekz. `limigita_string` → `limigita_teksto`) rajtas uzi `[skip ci]`, se la komita mesaĝo eksplicite notas, ke nur nomoj, komentoj kaj dokumentaro ŝanĝiĝis kaj ke la lokaj `fmt`, `Clippy` kaj testoj estis verdaj. La escepto ne validas por ŝanĝoj al la datumformato, la algoritmoj aŭ la DI-agordo.
- **Ĉiuj aliaj kodaj ŝanĝoj**: ajna ŝanĝo al kodo, testoj aŭ `.github/`, kiu ne estas pure dokumenta aŭ pure nomŝanĝo, devige rulas DI.
- **Kontrolo de rezultoj**: uzu la langeton **Actions** de la deponejo aŭ `gh run list` / `gh run watch`.
- **Difino de finiteco**: la laboro `Kontroli kaj konstrui` kaj ĉiuj laboroj `Kani (...)` devas esti verdaj en GitHub Actions.

