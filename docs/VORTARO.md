# VORTARO — Esperanto–angla terminaro

Ĉi tiu vortaro fiksas la terminologion de MiKerno kaj Versio. Novaj identigiloj,
komentoj kaj protokolaj nomoj de ambaŭ projektoj sekvu ĝin.

| Esperanto | Angla | Rimarko |
| --- | --- | --- |
| `procesoro` | processor / procesoro | Neniam uzu `cpu` en identigiloj. |
| `memoro` | memory | Neniam uzu `memory` aŭ `ram`. |
| `fadeno` | thread | Neniam uzu `thread`. |
| `procezo` | process | `Procezo`; neniam uzu `process`. |
| `IK` | IPC (interproceza komunikado) | La modulo `kerno/ik.rs`; antaŭe `ipc`. |
| `DI` | Continuous Integration (CI) | Daŭra Integrado: la laborfluo `di.yml` kaj la matrico `kani-kontroli`. |
| `KI` | command-line interface (CLI) | Komandliniaj Iloj; en Versio la dosierujo `versio-ki`. |
| `EAS` | content-addressable storage (CAS) | Enhavadresita Stokado; en Versio `libversio::stokado`. |
| `dosiera objekto` | blob | La kruda enhavo de unu dosiero; la tipo estas `Dosiero`. Neniam uzu `blob` en tekstoj aŭ identigiloj. |
| `branĉo` | branch | La koncepto; surdiske `.vs/referencoj/branĉoj`. La KI-komandoj restas `linio` kaj `linioj` pro la ASKIA limo. |
| `linio` / `linioj` | branch command names | ASKIAJ komandnomoj por krei kaj listigi branĉojn; la prozo uzas `branĉo`. |
| `KAPO` | HEAD | La dosiero `.vs/KAPO`; neniam uzu `HEAD` en identigiloj. |
| `laborspaco` | working tree | La dosierujo ekster `.vs`; `stato` komparas ĝin kun `KAPO`. |
| `laborspaca indekso` | work-tree index | `.vs/indekso`: metadatumkaŝmemoro por `stato` kaj `komitu`. |
| `eltiri` | checkout / restore | `vs eltiri <branĉo|haketo>`; restarigas la laborspacon. |
| `kunfandi` | merge | `vs kunfandi <branĉo>`; tri-voja kunfando. |
| `momentfoto` | snapshot | La arbo plus la komitaj metadatumoj; ne konfuzu kun `bildo`. |
| `seriigi` / `malseriigi` | serialize / deserialize | `Komito::seriigi`, `Legilo`; la surdiska kadro. |
| `movhistorio` | reflog | Fazo 5: la movoj de `KAPO` kaj de branĉoj. |
| `rubokolekti` | garbage collect | Fazo 5: forigi neatingeblajn objektojn. |
| `komito` | commit | `Komito`; la komando `vs komitu`. Neniam uzu `commit`. |
| `tirpeto` | pull request (PR) | `pull_request` restas kiel la nomo de la GitHub-events. |
| `Cargo-laborloko` | Cargo workspace | La radika `[workspace]` en `Cargo.toml`. |
| `kesto` | crate | `crate` restas kiel la Rust-ŝlosilvorto. |
| `pruvo` | Kani harness / proof | Unu funkcio markita per `#[kani::proof]`; la matrico `kani-kontroli`. |
| `statika analizo` | lint | `cargo clippy`; neniam uzu `lint` en tekstoj. |
| `Eldono` | GitHub Release | Publikigita versio kun la ISO-aktivaĵoj. |
| `aktivaĵo` | release asset | Dosiero alŝutita al Eldono. |
| `interfaco` | interface / API | `API` restas nur en eksteraj protokolaj nomoj. |
| `etendiana` | little-endian | La malalta bajto unue; 512-bajtaj sektoroj de SNDS. |
| `grandendiana` | big-endian | La alta bajto unue; la `fw_cfg`-registroj. |
| `ĉefmemoro` | RAM | Neniam uzu `ram` en identigiloj; `ramfb` restas protokola nomo. |
| `proceza identigilo` | process id | `ProcezaId`; `0` estas la nukleo; en `RSI` ĉe la Ring 3-eniro. |
| `proceza stato` | process state | `ProcezaEstado`: `Preta`, `Kuranta`, `Mortinta`. |
| `privata` | private | Proceza fenestro, kiun aliaj procezoj ne vidas. |
| `kunhavigi` | share | Kernaj mapoj restas komunaj inter procezoj. |
| `planisto` | scheduler | Neniam uzu `scheduler`. |
| `stato` | status | `stato()`, `agordi_staton`; antaŭe `status`. |
| `ŝanĝi` | switch | `Planisto::ŝanĝi`: kuntekstŝanĝo inter du fadenoj. |
| `ŝovo` | offset | Lokaj variabloj kaj adresaj kalkuloj. |
| `prioritato` | scheduling priority | `Prioritato::{Malalta, Normala, Alta}`; novaj fadenoj defaŭlte estas `Normala`; nur la kerno fiksas la prioritaton. |
| `tempopeco` | time slice / quantum | `Prioritato::tempopeco`: 1/2/4 temporizilaj tikoj; pezita cirkla planado, ne absoluta prioritata antaŭeco. |
| `restaj_tikoj` | remaining quantum ticks | `Planisto.restaj_tikoj`; cedo fordonas la restaĵon, nova elekto rekomencas la tempopecon. |
| `elekti_por_tiko` | tick selection | Elektas nur ĉe fino de tempopeco aŭ eksplicita cedo; `planumi_kun_cedo` plenumas la eventualan kuntekstŝalton. |
| `prioritataj provoj` | priority scheduling checks | `testa_planisto`, `prioritata_provo`, `maksimuma_atenda_provo`, `egaleca_provo`, `ceda_provo`, `bloka_provo`; `testi_prioritatojn` raportas la kvin fiksajn scenarojn; la DI-pruvoj estas en `kani_prioritatoj`. |
| `maksimuma preta atendo` | maximum ready wait | `MAKSA_PRETA_ATENDO = 28` tikoj por konstante preta fadeno: sep aliaj tempopecoj de maksimume kvar tikoj, sub regulaj planistaj vokoj. |
| `antaŭokupa provo` | preemption test | `antaŭokupa_provo`, `ANTAŬOKUPAJ_PRETOJ`, `ANTAŬOKUPA_KOMENCO`: kvar testfadenoj trapasas barilon sen cedo kaj observas tikojn dum komuna 32-tika intervalo. |
| `paĝo` | page | Tekstoj uzas `paĝ`; la modula dosiero estas `pagado.rs`. |
| `paĝado` | paging | `mod paĝado;` postulus ne-ASKIIan dosiernomon, kiun la aŭtomata modulŝarĝo rifuzas (E0754); ni elektis la ASKIIan `pagado`. |
| `kadro` | frame | Neniam uzu `frame`. |
| `kadra alokilo` | frame allocator | `KadraAlokilo`. |
| `virtuala adreso` | virtual address | |
| `fizika adreso` | physical address | |
| `adresa spaco` | address space | `AdresaSpaco`. |
| `radika adreso` | root address | Fizika adreso de `PML4` (`CR3`) aŭ de la radika `L0`-tabelo (`TTBR0_EL1`). |
| `radiko-ero` | root entry | `radiko_ero(cr3)` eltiras bitojn 51:12; la malaltaj 12 bitoj estas nulaj, ĉar `PCIDE` estas malŝaltita. |
| `aktivigi` | activate | Ŝalti la procezan `CR3` aŭ `TTBR0_EL1` kaj `TSS.RSP0` antaŭ kuntekstŝanĝo. |
| `paĝa tabelo` | page table | `PaĝaTablo`. |
| `paĝa eraro` | page fault (#PF) | |
| `datena aborto` | data abort | `ESR_EL1` EC `0x24`/`0x25`; EL0-aliro al folio sen la postulata permeso mortigas la procezon. |
| `instrukcja aborto` | instruction abort | `ESR_EL1` EC `0x20`/`0x21`; traktata kiel `ProtektaEraro`. |
| `ĝenerala protekta eraro` | general protection fault (#GP) | |
| `interrompo` | interrupt | |
| `interrompa peto` | interrupt request (IRQ) | Neniam uzu `IRQ` en identigiloj. |
| `interrompa kadro` | interrupt frame | `InterrompaKadro`. |
| `interrompa traktilo` | interrupt handler | `InterrompaTraktilo`. |
| `interrompa dispetĉo` | interrupt dispatch | `interrompa_dispetcho`. |
| `temporizilo` | timer (PIT) | Anstataŭ `PIT` en interna kodo. |
| `temporizila tiko` | timer tick | |
| `kontrolilo` | controller (APIC) | `apic` restas kiel modula nomo. |
| `tutmonda priskriba tabelo` | global descriptor table (GDT) | `TutmondaPriskribaTabelo`. |
| `interrompa priskriba tabelo` | interrupt descriptor table (IDT) | `InterrompaPriskribaTabelo`. |
| `taska stata segmento` | task state segment (TSS) | `TaskaStataSegmento`. |
| `priskribo` | descriptor | |
| `konservado` | storage | Modulo `konservado`; blokaj legado kaj skribado. |
| `bloko` | block | 512-bajta unuo (`BLOKA_GRANDO`). |
| `sektoro` | sector | Numero de bloko ĉe la aparato. |
| `virtio-blk` | virtio block device | Protokola nomo restas angla. |
| `transporto` | transport | `VirtioTransporto`; `VirtioSpeco::{Pcia, Mmio}`. |
| `trajto` | feature | `legi_trajtojn`, `agordi_trajtojn`. |
| `virtio-vico` | virtqueue | `Vico`: priskribiloj, disponebla ringo, uzita ringo. |
| `priskribilo` | descriptor | `Priskribilo`; ankaŭ GDT/IDT-priskriboj. |
| `disponebla ringo` | available ring | `Disponebla`. |
| `uzita ringo` | used ring | `Uzita`, `UzitaEro`. |
| `sciigi` | notify | `sciigi(vico)`; vica pordo, ne interrompo. |
| `kapablo` | capability | `Kapo`, `Kapoj`; PCI-kapablo. |
| `SNDS` | Simple Read-Only File System | Simpla NurLega Dosiersistemo; magio `SNDS`, versio 1, 512-bajtaj sektoroj. |
| `dosiersistemo` | file system | `SimplaDosiersistemo` en `konservado/snds.rs`. |
| `superbloko` | superblock | Sektoro 0: magio, versio, grandoj, limoj kaj radika inodo. |
| `inodo` | inode | 16 bajtoj: tipo, grando, unua sektoro, sektornombro. |
| `dosieruja ero` | directory entry | 64 bajtoj: inoda indekso + 60-bajta nomo kun fina NUL. |
| `datumbloko` | data block | Kontiguaj 512-bajtaj sektoroj de dosiero aŭ dosierujo. |
| `nurlega` | read-only | SNDS ne subtenas skribon aŭ kreadon de dosieroj. |
| `munti` | mount | `snds::pravalorizi()` validigas la superblokon kaj memprovas la bildon. |
| `SNDS-bildo` | SNDS image | `target/mikerno.snds`, kreata de `tools/krei_snds_img.sh`. |
| `selektilo` | selector | |
| `pordego` | gate | |
| `interrompa pordego` | interrupt gate | Malŝaltas `RFLAGS.IF`. |
| `kapta pordego` | trap gate | Ne malŝaltas `RFLAGS.IF`. |
| `privilegia nivelo` | privilege level (DPL/RPL/CPL) | |
| `kerna reĝimo` | kernel mode (Ring 0) | |
| `uzanta reĝimo` | user mode (Ring 3) | |
| `uzantspaco` | user space | |
| `sistemvoko` | system call | `SYS_*` restas kiel ABI-prefikso. |
| `sistemvoka vektoro` | syscall vector | `0x80`. |
| `sistemvoka numero` | system call number | `rax` (`x86_64`) aŭ `x8` (`arm64`) ĉe la voko. |
| `sistemvoka dispetĉo` | system call dispatch | `sistemvoka_dispetĉo`. |
| `uzanta montrilo` | user pointer | Kontrolata antaŭ ĉiu aliro, folio post folio. |
| `anoncita regiono` | declared region | La bildo kaj la uzanta stako. |
| `uzanta fenestro` | user window | `UZANTA_BAZO..UZANTA_FINO`. |
| `U-bito` | U bit | Ring 3-alireblo; tra ĉiuj niveloj. |
| `segmento` | segment (ELF `PT_LOAD`) | Malunue kun la GDT-segmentoj. |
| `programa kaplinio` | program header | ELF64 `Elf64_Phdr`. |
| `uzanta kodo` | user code | DPL3. |
| `uzantaj datumoj` | user data | DPL3. |
| `kerna stako` | kernel stack | Po procezo, 16 KiB; `RSP0` montras al ĝia pinto. |
| `uzanta stako` | user stack | |
| `uzanta fadeno` | user thread | Plenumas en Ring 3 post `iretq`. |
| `kunteksto` | context | `Kunteksto`. |
| `kunteksta ŝanĝo` | context switch | |
| `enirpunkto` | entry point | |
| `bildo` | image | Ekz. `horizonto`. |
| `modulo` | module | Multiboot2 module. |
| `praŝarĝa modulo` | boot module | Multiboot2 tipo 3 aŭ PVH-modlisto. |
| `modlisto` | module list (PVH) | `modlist_paddr`. |
| `praŝarĝilo` | bootloader | |
| `praŝarĝo` | boot | |
| `pravalorizi` | initialize | |
| `halti` | halt | |
| `atendi interrompon` | wait for interrupt | `hlt`. |
| `ebligi` / `malebligi` | enable / disable | |
| `mesaĝo` | message | `Mesago`. |
| `mesaĝujo` | mailbox / message queue | |
| `finpunkto` | endpoint | `FinaPunkto`. |
| `kapablo` | capability | `Kapablo`. |
| `permeso` | permission | `Permesoj`: `SENDI`, `RICEVI`, `REVOKI`, `ĈIUJ`, `PLENAJ`. |
| `tenilo` | handle | `Tenilo`: 8 bitoj da indekso + 24 bitoj da generacio; `0` estas malvalida. |
| `generacio` | generation | 24 bitoj, komenciĝas ĉe `1`; altiĝas ĉe reuzo de fendo (kontraŭ-ABA). |
| `indekso` | index | `Tenilo::indekso`, `0..=15`; `Tenilo` rifuzas pli grandan valoron. En Versio la laborspaca indekso estas aparta koncepto (vidu supre). |
| `finpunkta objekto` | endpoint object | `FinaPunktoObjekto` (Fazo 2); posedas la vicon kaj la atendoliston. |
| `revoki` | revoke | `REVOKI` (8): ĉiam tutfinpunkta (C14); `rsi` devas esti `0`; finpunkta generacio +1, ĉiuj teniloj malvalidiĝas; blokita ricevanto vekiĝas per `-32`. |
| `revokita` | revoked | `IkEraro::Revokita`: la finpunkto revokiĝis aŭ ĝia posedanto finiĝis; `-32`. |
| `revoka veko` | revocation wake | `REVOKAJ_VEKOJ[fadeno]`: unufoja marko por blokita ricevanto; `ricevi_kapable`/`ricevi_mesagon` ĝin konsumas kaj redonas `-32`. |
| `revoki la defaŭltan finpunkton` | revoke the default endpoint | `revoki_defaŭltan(fadeno)`: kerna vojo sen kapabla kontrolo (C8). |
| `forigi finpunkton` | remove endpoint | `forigu_finpunkton(fendo)`: liberigas ĉiujn kongruajn kapablofendojn en ĉiuj tabeloj, la finpunktan objekton, kaj vekas la ricevanton. |
| `purigi procezon` | clean up a process | `purigi_procezon(procezo)`: ĉe `SYS_FINI`, mortiga escepto kaj prepar-eraroj; liberigas la kapablojn kaj revokas la posedatajn finpunktojn. |
| `detrui uzanton` | destroy a user process | `detrui_uzantan(id)`: `purigi_procezon` + `procezo::detrui`. |
| `kodo de IK-eraro` | IPC error code | `IkEraro::kodo()`: `-1/-3/-4/-9/-28/-32/-90`; la sistemvoko nur konvertas ĝin al `u64`. |
| `akordi` | match | `akordas(aktiva, objekta_generacio, kapabla_generacio)`: la dua validiga nivelo; revoko aŭ emeritiĝo malakordigas ĝin. |
| `doni` | grant | `DONI` (7): la celo ricevas propran `Tenilo`; la petitaj permesoj devas esti subaro de la fontaj kaj la fonta kapablo restas valida. |
| `fermi` | close | `FERMI`: fermas unu lokan kapablofendon. |
| `finpunkta identigilo` | endpoint id | `FinaPunktaId` (`u64`); defaŭltaj finpunktoj uzas la fadenan id, eksplicitaj komenciĝas ĉe `UNUA_FINPUNKTA_ID = 16`. |
| `defaŭlta finpunkto` | default endpoint | Implica finpunkto de fadeno; `registri_fadenon` tenas ĝin vivan por la malnova vojo. |
| `posedanto` | owner | `FinaPunktoObjekto.posedanto`: la proceza id (`0` = kerno). |
| `referenckalkulo` | reference counting | `referencoj: u16`; saturiĝas anstataŭ subflui, kaj la objekto liberiĝas ĉe `0`. |
| `emerita` | retired | `FinaPunktaStato::Emerita`: la generacio eluziĝis, la fendo neniam plu atribuiĝas (C12). |
| `kapablotabelo` | capability table | `KAPABLOJ: [[KapabloEniro; 16]; 8]`; linio `0` (kerno) restas ĉiam malplena (C8). |
| `kapablofendo` | capability slot | `KapabloEniro`: `tenilo`, `permesoj`, `finpunkto`, `fp_generacio`, `uzata`. |
| `kapabla fenda generacio` | capability slot generation | `Kapablotabelo.generacioj[fendo]`: `0` = emerita; la `Tenilo` portas ĝin. |
| `akiri kapablon` | acquire capability | `akiri_kapablon(procezo, tenilo, bezonataj)`: unua nivelo (fendo + tenilo + permesoj), dua nivelo (aktiva finpunkto kaj `fp_generacio`). |
| `rajtoj-subaro` | rights subset | `dono_permesita`: `petitaj & !fontaj == 0`; malobservo redonas `-1`. |
| `permesa malakcepto` | permission denial | La kapablo validas sed mankas la bito: `-1` (ne `-9`). |
| `komenca tenilo` | bootstrap handle | `KOMENCA_TENILO = 0x100`: fendo `0`, unua generacio; transira distribuo antaŭ M17. |
| `ricevanto` | receiver | `FinaPunktoObjekto.ricevanto`: la fadeno blokita en `RICEVI` sur la finpunkto; la sendinto ĝin vekas. |
| `sendantoj` | blocked senders | `FinaPunktoObjekto.sendantoj: u16`: bitmasko de la fadenoj blokitaj en `SENDI` pro plena vico; bito `id − 1`. |
| `senda bito` | sender bit | `senda_bito(fadeno) = 1 << (fadeno − 1)`; la vek-buklo traktas la plej malaltan biton kiel fadenon `1`. |
| `premrego` | back-pressure | La sendanto atendas lokon en plena vico anstataŭ perdi la mesaĝon; nombrata en `IkStatistiko.premregoj`. |
| `sendblokado` | send blocking | `SENDI` blokas ĝis `preni_de` liberigas lokon aŭ la finpunkto revokiĝas (`-32`); nur la ripoza kunteksto ricevas `-4`. |
| `senda decido` | send decision | `senda_decido(enmetitaj, maks, povas_bloki)`: pura funkcio kun `Enmetu`/`Atendu`/`Rifuzi`, pruvita per Kani. |
| `IK-statistiko` | IPC statistics | `IkStatistiko`: `senditaj`, `ricevitaj`, `premregoj`, `revokoj`; `nuna_statistiko()` redonas kopion. |
| `registri fadenon` | register thread | `registri_fadenon(fadeno, procezo)`: idempotenta; rezervas la defaŭltan finpunkton kaj instalas la komencan tenilon en fendon `0`. |
| `krei finpunkton` | create endpoint | `KREI_FINPUNKTON` (6); redonas novan `Tenilo`. |
| `vivas` | is alive | `planisto::vivas(id)`: la celo ekzistas kaj ne estas `Mortinta`; la shim rifuzas per `-3`. |
| `tempolimo` | timeout | `RICEVI_TEMPO`/`SENDI_TEMPO`; `TEMPOLIMO = -110`. |
| `atendo` | wait mode | `Atendo::{Senfina, Nebloka, Tempa(tikoj)}`; `el_tempolimo(r10)` faras la decidon (C15). |
| `senfina tempolimo` | infinite wait | `SENFINA_TEMPOLIMO = u32::MAX`; `Atendo::Senfina` blokas ĝis mesaĝo aŭ revoko (`-32`). |
| `nebloka` | non-blocking | `r10 = 0` → `Atendo::Nebloka`; malplena vico redonas `PROVU_DENOVE = -11` sen cedi. |
| `tempolima politiko` | timeout policy | `el_tempolimo`: `(r10 as u32).div_ceil(10)` tikoj; 1–9 ms restas unu tiko, neniam nulo. |
| `absoluta limdato` | absolute deadline | `absoluta_limdato(atendo)` fiksas la limdaton ĉe la voko; `bloki_ĝis_absoluta` ne rekalkulas ĝin en la atenda buklo. |
| `tempolima veko` | timeout wakeup | `venkigi_tempolimojn()` elprenas el `vekoj` ĉiujn limdatojn `≤ nuna tiko`; `preni_tempolimon()` konsumas la markon; `IkStatistiko.tempolimoj`. |
| `vekotemporizilo` | wake timer | `Vekotemporizilo` en `kerno/vekotemporizilo.rs`: min-kupo de `(limdato, FadenoId)` kun pozicia tabelo; enigo, forigo kaj elpreno estas O(log n) sen alokado. |
| `veka kupo` | wake heap | La kupo mem: unu ero por fadeno, kapacito `MAKSAJ_FADENOJ`; `pinti` montras la plej fruan limdaton. |
| `dormi` | sleep | `SYS_DORMI` (12), `rdi` = daŭro en ms; `planisto::dormi` atendas `ceil(ms/10)` tikojn; la ripoza kunteksto ricevas `-1`. |
| `planista statistiko` | scheduler statistics | `PlanistaStatistiko`: `ŝaltoj`, `limdataj_blokoj`, `vekigoj`, `maks_vekprokrasto`. |
| `plenumtikoj` | sampled execution ticks | `Planisto.plenumtikoj` kaj `planisto::plenumtikoj()`: po-fadena saturiĝanta nombrilo; unu specimeno por ricevita temporizila interrompo, atribuita antaŭ ŝalto; indekso 0 inkluzivas ripozon kaj praŝarĝon. |
| `specimena plenumtempokontado` | sampled execution-time accounting | Nominale 10 ms/specimeno sur ambaŭ arkitekturoj; ne preciza murhorloĝa tempo, nek apartigo de uzanta/kernreĝima tempo; cedo ne kreas specimenon. |
| `plenumtempaj provoj` | execution-accounting checks | `testi_plenumtempon`, `plenumtempa_provo`, `ceda_kontada_provo`, `ripoza_kontada_provo`; `kani_plenumtempo` ankaŭ kontrolas arbitran nombrilon, apartigon kaj saturiĝon. |
| `samaj_plenumtikoj` | element-wise accounting comparison | Komparas du nombrotabelojn elementon post elemento anstataŭ per tuta-tabela egaleco, por ke Kani ne devu malvolvi la kompilan 72-bajtan `memcmp`; la asertoj restas samaj. |
| `veturoj` | scheduler runs | `Planisto.veturoj[fadeno]`: kiom da fojoj ĉiu fadeno estis elektita; la fina raporto komparas la minimumon por pruvi, ke neniu malsatiĝas. |
| `limdata bloko` | timed block | `limdataj_blokoj`: blokado kun limdato (dormo aŭ IK-tempolimo). |
| `maksimuma vekprokrasto` | maximum wakeup delay | `maks_vekprokrasto`: la plej granda distanco inter la limdato kaj la efektiva veko, en tikoj. |
| `fonta filtro` | source filter | `RICEVI_TEMPO.rdx`: fadena id aŭ `0` (ajna); `ricevi_mesagon_atenda(fonto, atendo)` preterlasas fremdajn sendintojn. |
| `ricevi kun tempolimo` / `sendi kun tempolimo` | receive / send with timeout | `ricevi_kun_tempolimo(tenilo, celo, fonto, ms)`, `sendi_kun_tempolimo(tenilo, mesago, ms)`. |
| `restarigi procezon` | reset process tables | `inicialigi_procezon(id)`: la kapablotabelo estas statika laŭ identigilo, do `krei` restarigas ĝin por ke `KOMENCA_TENILO` validu ankaŭ post reuzo. |
| `ABI-versio` | ABI version | `ABI_VERSIO = 2` ekde M14 Fazo 1. |
| `versio de la pakaĵo` | package version | `env!("CARGO_PKG_VERSION")`: la saluta linio legas ĝin dum kompilado. |
| `kontroli version` / `ĝisdatigi version` | check / update the version | `tools/versio.sh --kontroli` / `--ĝisdatigi`; la sola fonto estas `Cargo.toml`. |
| `nevalida tenilo` | invalid handle (`EBADF`) | `NEVALIDA_TENILO = -9`. |
| `provu denove` | try again (`EAGAIN`) | `PROVU_DENOVE = -11`. |
| `tabelo plena` | table full (`ENOSPC`) | `TABELO_PLENA = -28`. |
| `finpunkto revokita` | endpoint revoked (`EPIPE`) | `FINPUNKTO_REVOKITA = -32`. |
| `tro longa mesaĝo` | oversize message (`EMSGSIZE`) | `TRO_LONGA_MESAGO = -90`; `IkEraro::TroLongaMesago`; ekde Fazo 5 `SENDI` ĝin redonas, se `Mesago.longo > MAKSA_MESAGA_LONGO`. |
| `folia permeso` | leaf permission | `permesoj(adreso)` legas la realajn PTE-bitojn de folio. |
| `W^X` | write-xor-execute | Skribebla paĝo neniam estas plenumebla samtempe. |
| `sekcia limo` | section boundary | `__kerna_kodo_komenco`, `__kerna_rodata_fino`, … el `linker.ld`. |
| `antaŭkodo` | preamble | La regiono antaŭ `.text` (ELF-kapo kaj aligned-padding). |
| `nurlegaj datumoj` | read-only data | `.rodata`; R+NX. |
| `skribprotekto` | write protection | `CR0.WP`; Ring 0 ankaŭ respektas nurlegajn foliojn. |
| `kerna firmigo` | kernel hardening | M13: SMEP, SMAP, KASLR. |
| `procesoraj trajtoj` | processor features | `procesoro::trajtoj()`; CPUID folio 7. |
| `SMEP` | supervisor mode execution prevention | `CR4.SMEP`; Ring 0 ne plenumas `U=1`-foliojn (0.3.0). |
| `SMAP` | supervisor mode access prevention | `CR4.SMAP`; `STAC` malfermas kaj `CLAC` fermas la fenestron (0.3.1). |
| `PCID` | process-context identifier | `CR4.PCIDE` (bito 17); la praŝarĝilo povas lasi ĝin ŝaltita, sed `pagado::pravalorizi` ĝin malŝaltas (unue `CR3` kun nula PCID). |
| `ne PCID` | PCID disabled | `procesoro::ne_pcide()`; `ŝalti_radikon` haltas, se `PCIDE` restus ŝaltita. |
| `relokebla` | relocatable | KASLR postulas relokeblan aŭ alta-duonan kernon. |
| `alta duonmapo` | higher-half mapping | La kerno loĝas super la uzanta fenestro. |
| `hazarda bazadreso` | randomized base address | KASLR elektas ĝin ĉe starto. |
| `aparata izolado` | device isolation | IOMMU malhelpas arbitran DMA. |
| `DMA-mapo` | DMA mapping | IOMMU-donita traduko por aparato. |
| `interrompa readresado` | interrupt remapping | IOMMU-akompananto por interrompoj. |
| `STAC-fenestro` | STAC window | `StacGardo`: mallonga Ring 0-alirejo al uzantaj paĝoj; `Drop` fermas ĝin per `CLAC`. |
| `AC-bito` | AC flag | `STAC` starigas, `CLAC` nuligas por SMAP. |
| `AC-provo` | AC probe | `firmigo::testi_ac()`: fermita, malfermita, nestita, restarigita, ŝtelita kaj purigita fenestro. |
| `AC-restaĵo` | residual AC | Se `SMAP` estas aktiva kaj `AC` restas post `malŝalti_ac`, la interrompa dispetĉo haltas. |
| `gardoprovo` | guard probe | `testi_gardojn()`: `InterrompaGardo` malŝaltas `IF`, kaj `Drop` restarigas la antaŭan staton ankaŭ ĉe nestitaj gardoj. |
| `sendi` / `ricevi` | send / receive | |
| `bloki` / `veki` | block / wake | |
| `cedi` | yield | `SYS_CEDI`. |
| `morti` / `fini` | exit | |
| `unuopa` | uniprocessor | |
| `antaŭokupa` | preemptive | |
| `cirkla vico` | circular queue | |
| `kapo` / `vosto` | head / tail | |
| `bitmapo` | bitmap | |
| `folio` | leaf page | |
| `ero` | entry | |
| `limo` | limit | |
| `bazo` | base | |
| `nuligi` | zero / clear | |
| `liberigi` | free / release | Redoni kadron al la alokilo; duobla liberigo estas senefika. |
| `detruu` | destroy | Malmapu kaj liberigu la tutan adresan spacon. |
| `detruo` | destruction / teardown | La substantivo de `detruu`; `detrui` estas la transitiva formo. |
| `privata PDPT` | private PDPT | Kampo `privataj_pdptoj`; trajta aliro `privataj_subtabeloj`. |
| `reuzo` | reuse | `testi_procezan_reuzon`; la alokilo reuzas liberigitajn kadrojn. |
| `ciklo` | cycle | Unu krea-detrua ripeto en la reuzoprovo. |
| `komenca reĝimo` | start mode | `REĜIMO_SEKRETO`, `REĜIMO_SPIONO`; transdonata en `RDI`. |
| `sekreta paĝo` | secret page | `SEKRETA_ADRESO`; sama virtuala adreso, malsama kadro por procezo. |
| `dua sekreta paĝo` | second secret page | `SEKRETA_DUA_ADRESO`; nur en la viktima spaco. |
| `magia numero` | magic number | `MAGIA_NUMERO = 0x4D69_4B65_726E_6F21`. |
| `viktimo` | victim | Procezo `4`; mapas la duan paĝon kaj pluvivas. |
| `spiono` | spy | Procezo `5`; legas fremdan paĝon kaj ricevas `#PF` aŭ datenan aborton. |
| `spioni` | spy on | `spioni(id)` en `horizonto`. |
| `izolado` | isolation | `[memoro] izolado (bone)` komparas liberajn kadrojn. |
| `samtempeco` | concurrency | Ambaŭ procezoj pretaj antaŭ unu `planisto::lanĉi()`. |
| `interplektiĝi` | interleave | La du procezoj alterne plenumiĝas. |
| `elformata linio` | formatted line | Unu `SYS_SKRIBI` por linio, por eviti dispecetigon. |
| `malmapi` | unmap | Forigu eron el la paĝa tabelo. |
| `proviza` | scratch | Proviza spaco por memtesto. |
| `kopii` | copy | |
| `validigi` | validate | |
| `eraro` | error | |
| `sukceso` | success | |
| `rezulto` | result | |
| `argumento` | argument | |
| `registro` | register | |
| `stako` | stack | |
| `pinto` | top | |
| `kreski malsupren` | grow downward | |
| `frambufo` / `ekrano` | framebuffer / screen | `FrambufaInformo`; ankaŭ `ekrano` por la fenestra bildo. |
| `bitoj po pikselo` | bits per pixel | Kampo `bitoj_po_pikselo`; eligata kiel `bitoj/pikselo`. |
| `seria konzolo` | serial console | COM1 sur `x86_64`; PL011 sur `arm64`. |
| `tiparo` | font | |
| `glifo` | glyph | |
| `streko` | strike / font size | |
| `koloro` | color | |
| `pikselo` | pixel | |
| `kaŝmemoro` | cache | |
| `nekaŝebla` | uncacheable | PCD. |
| `plenumebla` | executable | NX. |
| `skribebla` | writable | |
| `ĉeesta` | present | |
| `grandega paĝo` | huge page | |
| `aliniiĝinta` | aligned | |
| `ŝpuro` | track | Aparta disvolva linio; la ARM-ŝpuro estis kunfandita en M15. |
| `platforma malantaŭo` | platform backend | `malantaŭo` en `arkitekturo`; `x86_64` aŭ `arm64`. |
| `interrompa kadro` | interrupt frame | `InterrompaKadro`: platforma tipo en `x86_64/interrompo.rs` aŭ `arm64/interrompo.rs`. |
| `sistemvoka kadro` | system-call frame | `SistemvokaKadro`; la komuna kodo legas la registrojn per ĝi. |
| `escepta speco` | exception kind | `EsceptoTipo`: `PaĝaEraro`, `ProtektaEraro`, `Nekonata`. |
| `instali sistemvokon` | install system calls | `instali_sistemvokon()`; la malantaŭo registras sian enirmekanismon. |
| `eniri uzanton` | enter user mode | `eniri_uzanton()`: `iretq` al Ring 3 aŭ `eret` al EL0. |
| `ELF-maŝino` | ELF machine | `ELF_MAŜINO`: `62` (`x86_64`) aŭ `183` (`arm64`). |
| `firmiga raporto` | hardening report | `FirmigaRaporto`; SMEP/SMAP aŭ PXN/UXN/PAN. |
| `privata subtabelo` | private subtable | Anstataŭas `privataj_pdptoj`: PDPT aŭ L1/L2-subtabeloj. |
| `antaŭeldono` | pre-release | `0.4.3-arm.N` dum la ARM-ŝpuro. |
| `CR3-provo` | CR3 probe | `testi_cr3(radiko)`: `PCID=malŝaltita`, la aktiva radiko egalas la konstruitan `PML4`, kaj ĝi estas paĝ-aliniiĝinta. |
| `EOI-nombrilo` | EOI counter | `KONFIRMOJ`/`konfirmoj_interrompoj()`: nombro de senditaj `EOI`-oj; la fina raporto komparas ĝin kun la temporizilaj tikoj. |
| `EOI-politiko` | EOI policy | `estas_apic_vektoro`: nur `0x20..0x80`; `int 0x80`, `int3` kaj la spuria `0xFF` restas ekstere. |
| `sen EOI` | no EOI | `sen_eoi` en la Ring 0-memtesto: `int 0x80` ne rajtas pliigi la `EOI`-nombrilon. |
| `ellasa revizio` | release review | Fazo 7: revizio de la ellasaj vojoj — `STAC`-gardo, `CR3`-ŝalto, `EOI` kaj liberigo/revoko. |
| `plata bildo` | flat image | `Image`: senŝela binara bildo kun 64-bajta kapo. |
| `Image-kapo` | Image header | `code0`/`code1`, `text_offset`, `image_size`, `flags`, magio `0x644d5241`. |
| `FDT` | flattened device tree | Analizilo `arm64/fdt.rs`; la praŝarĝilo transdonas la `DTB` en `x0`. |
| `DTB` | device tree blob | |
| `PSCI` | Power State Coordination Interface | `SYSTEM_OFF` per `hvc #0`. |
| `PL011` | ARM UART | Seria konzolo ĉe `0x0900_0000` sur QEMU `virt`. |
| `elstara nivelo` | exception level | `CurrentEL`; EL2→EL1 per `HCR_EL2.RW` kaj `eret`. |
| `identeca mapo` | identity map | Virtuala adreso egalas la fizikan. |
| `L0-tabelo` / `L1-tabelo` | level-0 / level-1 table | 4-nivela paĝado kun 1 GiB-blokoj. |
| `L2-tabelo` / `L3-tabelo` | level-2 / level-3 table | Privataj tabeloj por la uzanta fenestro (`UZANTA_BAZO..UZANTA_FINO`). |
| `SVC64` | SVC from AArch64 | `ESR_EL1` EC `0x15`; la vektora ŝelko ĝin dispetĉas al `kerno::sistemvoko`. |
| `EL0t` | EL0 with SP_EL0 | `SPSR_EL1 = 0` ĉe la unua `eret`; `SP_EL0` portas la uzantan stakon. |
| `bloka ero` | block entry | Priskribo kun `AF`, `SH`, `PXN`/`UXN`. |
| `kerna fino` | kernel end | `__kerna_fino`; eniras `image_size`. |
| `funkciovoko` | hypervisor call | `hvc #0` por PSCI. |
| `UEFI-ISO` | UEFI ISO | `mikerno-arm.iso`; El Torito + `BOOTAA64.EFI`. |
| `El Torito` | El Torito | Elŝuta speco de ISO9660; la firmvaro legas la EFI-bildon. |
| `UEFI-memormapo` | UEFI memory map | `/chosen/linux,uefi-mmap-*`; uzata kiam `/memory` mankas. |
| `konvena memoro` | conventional memory | UEFI-tipo 7; la «uzebla» kvanto en la raporto. |
| `AAVMF` | AArch64 UEFI firmware | `QEMU_EFI.fd`; uzata de `just ruli-iso-arm`. |
| `vektora tabelo` | vector table | `VBAR_EL1`; 16 eniroj po 128 bajtoj; `__vektoroj` en `arm64/interrompo.rs`. |
| `vektoro` | vector | La enira indekso 0–15 aŭ GIC-identigilo; `InterrompaKadro::vektoro`. |
| `distribuilo` | distributor | GIC-distribuilo (`GICD_*`) ĉe `0x0800_0000`. |
| `procesora interfaco` | procesoro interface | GICC (`GICC_*`) ĉe `0x0801_0000`. |
| `aparta aparata interrompo` | private peripheral interrupt (PPI) | PPI 30 estas la ĝenerala temporizilo. |
| `ĝenerala temporizilo` | generic timer | `CNTFRQ_EL0`, `CNTP_TVAL_EL0`, `CNTP_CTL_EL0`. |
| `tika frekvenco` | tick rate | `FREKVENCO = 100` (100 Hz, 10 ms). |
| `trampolino` | trampoline | `fadeno_trampolino`: la unua eniro de nova fadeno malmaskas `IRQ`. |
| `fadeno-stako` | thread stack | `FadenaStako`; 16 KiB po fadeno. |
| `stako-pinto` | stack top | `stako_pinto`; la enira stako finiĝas ĉe `__stako_supro`. |
| `enira stako` | entry stack | La sekcio `.stako` ekster `.bss`, do la nula buklo ne tuŝas ĝin. |
| `fw_cfg` | firmware configuration | La MMIO-aparato de QEMU (`qemu,fw-cfg-mmio`); dateno, selektilo kaj DMA. |
| `selektilo` | selector | La 2-bajta grandendiana registro ĉe `fw_cfg + 0x08`. |
| `dosierujo de fw_cfg` | fw_cfg file directory | Klavo `0x0019`; eroj po 64 bajtoj kun nomo kaj selektilo. |
| `ramfb` | RAM framebuffer | La `fw_cfg`-dosiero `etc/ramfb`; `RAMFBCfg` 28 bajtoj. |
| `kvarobla signo` | fourcc | `XR24` (`0x3432_5258`): XRGB8888 kun 32-bitaj pikseloj. |
| `paŝo` | stride | Bajtoj po vico; `LARĜO * 4 = 5120`. |
| `DMA-priskribo` | DMA descriptor | `DmaEro`: `kontrolo`, `longo`, `adreso`, ĉiuj grandendianaj. |
| `kontiguaj kadroj` | contiguous frames | `aloki_kontiguajn(n)`: unua-taŭga serĉo kaj markado de `n` sinsekvaj kadroj. |
| `piksela fiksaĵo` | pixel fixture | `testi_pikselojn`: gradiento, teksto, haketo; `PikselaRaporto`. |
| `haketo` | hash | FNV-1a 64 super la videblaj pikseloj; komparebla inter la arkoj. |
| `PXN` | privileged execute-never | PTE-bito: `EL1` ne plenumas `U=1`-foliojn. |
| `UXN` | unprivileged execute-never | PTE-bito: `EL0` ne plenumas datumajn foliojn; la kodo havas `UXN = 0`. |
| `PAN` | privileged access never | `ID_AA64MMFR1_EL1`; `msr pan, #0` malfermas, `#1` fermas la fenestron. |
| `SPAN` | set PAN | `SCTLR_EL1.SPAN = 0`: ĉiu escepta eniro refermas la `PAN`-fenestron. |
| `PAN-fenestro` | PAN window | `StacGardo` (arm64): malfermas ĉe profundo 0 kaj refermas per `Drop`. |
| `PAN-malobservo` | PAN violation | `EC = 0x25` kun `FAR` en la uzanta fenestro; la kerno haltas. |

## FluoU — grafika uzinterfaco

| Esperanto | Angla | Rimarko |
| --- | --- | --- |
| `FluoU` | FluoU | Propra nomo: Fluo + Uzinterfaco; la deviga GUI-kadro de Horizonto. |
| `uzinterfaco` | user interface | Neniam uzu `UI` en identigiloj. |
| `grafika uzinterfaco (GU)` | graphical user interface (GUI) | Nur `FluoU` rajtas ĝin provizi. |
| `fenestro` | window | Ĉiu fenestro posedas `FENESTRO`-tenilon. |
| `titolstrio` | title bar | 32 DIP alta; samtempe trenareo. |
| `fenestra regilo` | caption button | Tri: malgrandigi, pligrandigi/restarigi, fermi. |
| `malgrandigi` | minimize | Windows-modela regilo. |
| `pligrandigi` | maximize | La ikono ŝanĝiĝas al `restarigi`. |
| `restarigi` | restore | Revenigas la antaŭan grandecon. |
| `fermi` | close | Fenestra ago; ne konfuzu kun `FERMI` de kapabloj. Ruĝa nur en la ŝveba stato de la ferma regilo. |
| `trafiklumo` | traffic light | La macOS-regilo; malpermesita en FluoU. |
| `Mito` | Windows key | La tradicia `Win`-klavo; la efektiva klavkombino de Eniro. `Ctrl+E` estas nur provizora solvo. |
| `komponanto` | component | Publika interfaco de FluoU. |
| `ĵetono` | design token | Koloro, mezuro, movo aŭ denseco. |
| `materialo` | material | Nur `Mika`, `Akrila` kaj `Revelo`. |
| `Mika` | Mica | Propra materiala nomo; netravidebla bazo. |
| `Akrila` | Acrylic | Propra materiala nomo; fona nebulo. |
| `Revelo` | Reveal | Propra materiala nomo; montrilsekva brilo. |
| `kompozitoro` | compositor | Izolita servo; sola aliras la frambufon. |
| `akcento` | accent | Unu el ses antaŭagordoj aŭ propra valoro. |
| `temo` | theme | `malluma` aŭ `hela`. |
| `denseco` | density | `kompakta`, `norma` aŭ `komforta`. |
| `elevacio` | elevation | Ombraj niveloj `0`–`4`. |
| `radiuso` | corner radius | Defaŭlte 8 DIP. |
| `grajno` | grain | Subtila brua teksturo. |
| `komanda paletro` | command palette | Eniro. |
| `helpa surfaco` | auxiliary surface | Eniro kaj Ligo; titolstrio laŭ §3.5 de FLUOU.md. |
| `objekta ŝelo` | object shell | Ligo. |
| `objekta fluo` | object stream | Rekordoj, ne teksto. |
| `rekordo` | record | Unuo de la objekta fluo. |
| `kampo` | field | Kolumno de rekordo. |
| `stadio` | pipeline stage | `filtri`, `ordigi`, `montri`, … |
| `dukto` | pipeline | Sinsekvo de stadioj. |
| `projekcio` | projection | Elekti kolumnojn per `montri`. |
| `skalaro` | scalar | Rezulto de `sumi` aŭ `nombri`. |
| `fulmomesaĝo` | toast notification | Maksimume kvar samtempe. |
| `informa strio` | info bar | Kvar semantikoj; fermebla. |
| `langeto` | tab | |
| `ŝaltilo` | switch | |
| `glitilo` | slider | |
| `ŝildeto` | badge | |
| `tenila ŝildeto` | handle badge | Montras `TENILO`. |
| `protekta procezo` | protected process | `pid ≤ 3`; `mortigi` rifuzas. |
| `piksela egaleco` | pixel parity | Inter `x86_64` kaj `arm64`. |
| `aplikaja manifesto` | application manifest | Enhavas `gu = "fluoU"`. |
| `izolita aplikaĵo` | sandboxed application | Minimumaj kapabloj. |
| `kapabla renderado` | capability-based rendering | Sen tenilo ne ekzistas pikseloj. |
| `rekta frambufa aliro` | direct framebuffer access | Malpermesita al aplikaĵoj. |
| `vida montro` | visual demo | `docs/FLUOU-montro.html`; nur Esperanta, derivita el la ĵetonoj de tiu ĉi gvidilo. |
| `sceno` | scene | Sekcio de la vida montro: fenestro, Eniro, Ligo. |
| `enira tavolo` | palette overlay | Eniro levita super la paĝo per `Ctrl+E` (provizora klavo; estonte `Mito`). |
| `malantaŭa kurteno` | backdrop | Mallumigita fono sub la enira tavolo; klako fermas. |
| `retejo` | website | `docs/index.html`, `docs/retejo.css` kaj `docs/retejo.js`; la fonto estas Esperanta. |
| `reteja versio` | site version | `data-versio` sur la `<html>`-etikedo; kontrolata de `tools/versio.sh`. |
| `mapada tavolo` | mapping layer | Aparta traduka dosiero por montrataj tekstoj; ne anstataŭas la fonton. |
| `angla mapado` | English mapping | `docs/tradukoj/angla.js`; nur montrataj tekstoj, sen novaj identigiloj. |
| `reteja sceno` | site scene | Sekcio de la retejo, ekzemple `#kapabloj` aŭ `#fluou`. |

## Komandoj de Ligo

Ĉiuj komandnomoj estas Esperantaj; la anglaj formoj estas malpermesitaj, ankaŭ
kiel kaŝitaj aliasoj.

| Esperanto | Angla (malpermesita) | Signifo |
| --- | --- | --- |
| `helpu` | `help` | Montri la helpon |
| `listi` | `ls` | Listigi dosierojn |
| `legi` | `cat` | Legi dosieron |
| `procezoj` | `ps` | Montri procezojn |
| `mortigi` | `kill` | Mortigi procezon |
| `eldiri` | `echo` | Krei mesaĝan rekordon |
| `pri` | — | Sistemaj informoj |
| `temo` | `theme` | Ŝanĝi la temon |
| `akcento` | `accent` | Ŝanĝi la akcenton |
| `eniro` | — | Malfermi Eniron |
| `filtri` | `grep` | Filtri rekordojn |
| `ordigi` | `sort` | Ordigi rekordojn |
| `unua` | `head` | Preni la unuajn rekordojn |
| `sumi` | `sum` | Sumi kampon |
| `nombri` | `wc` | Nombri rekordojn |
| `montri` | — | Projekcii kolumnojn |
| `teksto` | — | Teksta limo |
| `purigi` | `clear` | Malplenigi la terminalon |

## Protokolaj kaj aparataraj nomoj

La jenaj nomoj restas en sia originala formo, ĉar ili estas aparataj aŭ
eksteraj protokolaj nomoj:

`x86_64`, `CR0`, `CR2`, `CR3`, `CR4`, `PCID`, `PCIDE`, `RSP0`, `RIP`, `RFLAGS`, `RAX`, `RDI`,
`RSI`, `RDX`, `R10`, `MSR`, `EFER`, `NXE`, `PAE`, `PML4`, `PDPT`, `PD`, `PT`,
`IDT`, `GDT`, `TSS`, `APIC`, `PIT`, `COM1`, `Multiboot2`, `Xen-PVH`, `ELF`,
`Limine`, `QEMU`, `int 0x80`, `iretq`, `syscall`, `sysret`, `fork`, `exec`, `execve`, `ASCII`, `UTF-8`, `W^X`, `CR0.WP`,
`SMEP`, `SMAP`, `KASLR`, `IOMMU`, `DMA`, `PCI`, `DMAR`, `IVRS`, `STAC`, `CLAC`,
`KREI_FINPUNKTON`, `DONI`, `REVOKI`, `RICEVI_TEMPO`, `SENDI_TEMPO`, `FERMI`,
`EBADF`, `EAGAIN`, `ENOSPC`, `EPIPE`, `EMSGSIZE`, `ETIMEDOUT`, `arm64`,
`AArch64`, `EL0`, `EL0t`, `EL1`, `EL2`, `SVC`, `SVC64`, `GIC`, `GICv2`,
`GICv3`, `PL011`, `FDT`,
`DTB`, `Image`, `TTBR0_EL1`, `TCR_EL1`, `MAIR_EL1`, `SCTLR_EL1`, `VBAR_EL1`,
`ESR_EL1`, `FAR_EL1`, `ELR_EL1`, `SPSR_EL1`, `DAIF`, `CNTFRQ_EL0`,
`CNTP_TVAL_EL0`, `CNTP_CTL_EL0`, `GICD_CTLR`, `GICD_ISENABLER`,
`GICD_IPRIORITYR`, `GICD_ITARGETSR`, `GICC_CTLR`, `GICC_PMR`, `GICC_IAR`,
`GICC_EOIR`, `PXN`, `UXN`, `PAN`, `PSCI`, `WFI`, `HCR_EL2`, `eret`, `svc`,
`fw_cfg`, `FW_CFG_FILE_DIR`, `qemu,fw-cfg-mmio`, `etc/ramfb`, `ramfb`,
`RAMFBCfg`, `XR24`, `FNV-1a`, `SNDS`, `virtio-blk`, `virtio-pci`, `virtio-mmio`,
`virtqueue`, `BAR`, `ID_AA64MMFR1_EL1`, `SCTLR_EL1.SPAN`, `msr pan`,
`cortex-a76`.

## Reguloj

1. Novaj tipoj, funkcioj, kampoj kaj lokaj variabloj uzu Esperantajn vortojn.
2. Ne uzu `cpu`, `ram`, `memory`, `thread`, `scheduler`, `page`, `frame` en
   identigiloj.
3. Komentoj kaj logaj tekstoj estas Esperantaj.
4. Kiam necesas mencii anglan mallongigon, skribu unue la Esperantan nomon kaj
   poste la mallongigon en parentezo: `la tutmonda priskriba tabelo (GDT)`.
5. Registraj nomoj, instrukcioj kaj eksteraj protokoloj restas netuŝitaj.
6. Ĉiuj komandnomoj de Ligo estas Esperantaj kaj sen diakritaj signoj; anglaj
   aliasoj estas malpermesitaj, ankaŭ kaŝe.
7. La fenestraj regiloj sekvas la Windows-modelon: dekstre, en la ordo
   `malgrandigi`, `pligrandigi`/`restarigi`, `fermi`. La titolo estas
   horizontale centrita, la aplika ikono staras maldekstre. La koloraj rondaj
   `trafiklumoj` de macOS estas malpermesitaj.
8. Ĉiuj koloroj, mezuroj kaj daŭroj venas el la ĵetonoj de FluoU; ne fiksu
   ilin rekte en komponantoj.
9. `docs/FLUOU-montro.html` estas nur Esperanta kaj spegulas la regulojn de
   `docs/FLUOU.md`; kiam la gvidilo ŝanĝiĝas, ĝisdatigu ankaŭ la montron.
10. La retejo `docs/index.html` uzas la samajn FluoU-ĵetonojn kiel la vidmontro;
    la angla mapado restas en `docs/tradukoj/angla.js` kaj ne rajtas ŝanĝi
    identigilojn, protokolajn nomojn aŭ komandnomojn.
11. La helpaj surfacoj `Eniro` kaj `Ligo` uzas 32-DIP-an titolstrian skemon:
    ikono maldekstre, titolo centre. `Ligo` estas vera fenestro, do ĝi montras
    la tri regilojn dekstre (stato antaŭ ili); `Eniro` ne havas fenestrajn
    regilojn kaj fermiĝas per `Esc` aŭ klako sur la malantaŭa kurteno.
12. Modulaj dosiernomoj restas ASKIAJ (`pagado.rs`), dum la prozo uzas la
    diakritojn («paĝado», «paĝa tabelo»). Ne-ASKIAj identigiloj kiel
    `PaĝaTablo` estas permesataj; la limo koncernas nur la aŭtomatan
    modulŝarĝon (E0754).
