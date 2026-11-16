# MiKerno-integro: Fazo 4

## Amplekso

La koncepta provo importas unu Git-momentfoton en novan Versio-deponejon.
Ĝi ne migras Git-historion, branĉojn, etikedojn aŭ aŭtorojn, kaj ne integras
Versio en la rulantan kernon de MiKerno. Neniu MiKerno-konstruado, QEMU
aŭ loka Kani-rulo estas bezonata.

`git archive` eksportas la elektitan komiton, ne la labordosierujon: `.git`,
nekomititaj ŝanĝoj kaj nesekvataj konstruaĵoj ne estas kopiitaj. Git-atributoj
kiel `export-ignore` kaj `export-subst` povas influi la eksporton; submoduloj
ne estas rekursive eksportataj. La testita momentfoto ne enhavis simbolajn
ligilojn aŭ submodulojn.

Ne rulu `vs iniciati` rekte en la origina Git-deponejo: nun Versio preterlasas
nur `.vs`, ne `.git`, kaj ne interpretas `.gitignore`. Ĝi konservas ordinarajn
dosierojn kaj arbojn, ne plenumeblajn permesbitojn aŭ simbolajn ligilojn.

## Ripetebla importo

Antaŭkondiĉoj: Git, tar kaj ekzistanta `vs`-binaro. La loka malpeza kontrolo
`cargo test --workspace --all-features` en Versio produktas `target/debug/vs`;
ankaŭ jam konstruita eldona binaro taŭgas. Agordu la du absolutajn vojojn.

```bash
set -euo pipefail
MIKERNO=/home/makoto/mikerno
VS=/home/makoto/versio/target/debug/vs
REV=$(git -C "$MIKERNO" rev-parse HEAD)
CELO=$(mktemp -d -t versio-mikerno-XXXXXX)
git -C "$MIKERNO" archive --format=tar "$REV" | tar -xf - -C "$CELO"
(
    cd "$CELO"
    "$VS" iniciati
    "$VS" stato
    "$VS" komitu --mesagxo "Importi MiKerno $REV"
    "$VS" stato
)
printf 'Git-komito: %s\nVersio-deponejo: %s\n' "$REV" "$CELO"
```

La unua `stato` listigu la eksportitajn dosierojn sub `Novaj:`; la dua
redonu `Neniu ŝanĝo`. `komitu` kaptas la tutan labordosierujon: aparta
`add`-komando ne ekzistas. La nova deponejo restas en `$CELO` por inspektado.

Por la ĉi-suba testita momentfoto, daŭrigu en la sama ŝelo. La ŝanĝoj tuŝas
nur la provizoran kopion; la forigita dosiero restas en la origina Git-komito.

```bash
(
    cd "$CELO"
    printf '\nVersio-integra provo.\n' >> README.md
    printf 'Nova dosiero por la integra provo.\n' > integra-provo.txt
    rm -- docs/VORTARO.md
    "$VS" stato
    "$VS" komitu --mesagxo "Kontroli modifon aldonon kaj forigon"
    "$VS" stato
)
```

## Registrita loka provo

Dato: 2026-11-16. Medio: Linux/WSL2, x86_64; loka `target/debug/vs`
rekonstruita per la unuoptestoj. La CLI-logiko estas tiu de Versio `68be503`;
la Fazo-4-ŝanĝoj aldonas pruvojn, testojn kaj dokumentadon.

- MiKerno Git-komito: `154e506211d3a94cc759f07fc9d101d694b14869`.
- Eksportitaj ordinaraj dosieroj: **99**, entute **5 106 098 bajtoj**.
- SHA-256 de la Git-tar-arkivo: `d74a5aa7a2adecbb31b8d6065937e980eecde175280592e35c6e02a61a8770c8`.
- Ĉiu elpakita dosiero estis bajt-post-bajte komparita kun sia arkiva enhavo.
- La unua `stato` estis ekzakte komparita kun la ordigita listo de ĉiuj 99 dosieroj.
- La fontaj `git status`, `git diff --binary` kaj `git diff --cached --binary`
  estis identaj antaŭ kaj post la provo. La originalo ne estis modifita.

Realaj eligoj post la unua komito:

```text
Komito 420b7a102a62eeec870e490c9b4b09805a2798f57f1995cdfaf69cf7f832b596
Neniu ŝanĝo
```

Post la tri ŝanĝoj, la tuta statuseligo estis ekzakte kontrolita:

```text
Modifitaj:
  README.md
Novaj:
  integra-provo.txt
Forigitaj:
  docs/VORTARO.md
```

Post la dua komito:

```text
Komito 105513f1a409fa962cab78867cf0006a12ecfe6abe863709b4fb6080050557c7
Neniu ŝanĝo
```

La du Versio-komitoj havis malsamajn identigilojn. La testprogramo finiĝis kun
`SOURCE_UNCHANGED=true` kaj `RESULT=PASS`. Komitaj identigiloj dependas ankaŭ
de tempstampo, do alia rulo ne devas reprodukti tiujn samajn identigilojn.

## Formala pruvado kaj ĝiaj limoj

La kvin antaŭaj Kani-pruvoj restas sen ŝanĝo de sia amplekso. Kvar novaj
sendependaj CI-laboroj uzas la verajn `Komito::seriigi` kaj
`Komito::malseriigi`, sen stumpoj aŭ novaj `kani::assume`:

| Pruvo | Preciza modelo kaj aserto |
| --- | --- |
| `pruvo_komito_seriigo` | Simbolikaj radika kaj gepatra haketoj, ambaŭ gepatro-branĉoj, mesaĝo de 0–1 ASCII-bajto kaj ajna `u64`-tempstampo. Seriigo devas sukcesi, la eliglongo resti inter 35 kaj 77 bajtoj, kaj la unuaj 32 bajtoj egali la radikan haketon. |
| `pruvo_komito_malseriigo` | Fiksa staka kadro (45 aŭ 77 bajtoj) kun konkreta longoprefikso, simbolikaj haketoj, ambaŭ gepatro-branĉoj, unu-bajta mesaĝo kaj ajna `u64`-tempstampo. Malseriigo devas redoni precize la atenditan komiton. |
| `pruvo_komito_nevalida_etikedo` | Fiksa 37-bajta sengepatra kadro; ĉiuj 256 gepatro-etikedoj. Nur la kompleta kadro kun etikedo 0 rajtas sukcesi. |
| `pruvo_komito_trancxo` | Fiksa 37-bajta sengepatra kadro; ĉiuj prefiksoj. Nur la kompleta kadro rajtas sukcesi. |

La permane konstruita kadro kaj la fiksa 10-bajta varinto evitas, ke Kani
traktu la longoprefikson kiel arbitran simbolon kaj esploru centojn da
UTF-8-longoj. La fiksa varinto estas valida `postcard`-kodigo por ajna `u64`
(kvankam ne la plej mallonga por malgrandaj valoroj). La kongruon de la kadro
kun `Komito::seriigi` fiksas `testo_komito_fiksa_kodajo`; la akcepton de la
fiksvarinta kadro kontrolas `testo_komito_akceptas_fiksan_tempstampan_varinton`.

`kani::cover!` kontrolas atingeblajn sukcesajn kaj erarajn kazojn. La malvolvaj
limoj estas respektive 40, 32, 16 kaj 16; la malvolvaj asertoj ne estas
malŝaltitaj. Ĉiu nova pruvo havas la saman 600-sekundan komandlimon kaj
30-minutan laborlimon kiel la aliaj. Neniu pruvo estas rulata loke.

Tio estas limigita modelo, ne pruvo por ĉiu arbitra komito, ĉiu UTF-8-mesaĝo,
ĉiu arbo aŭ por la kriptografia sekureco de BLAKE3. Pliaj unuoptestoj kovras
ambaŭ gepatro-kazojn, malplenajn/Unikodajn/maksimumlongajn mesaĝojn,
varintajn tempstampolimojn, ĉiujn verajn prefiksojn de du specimenaj komitoj,
postrestajn bajtojn kaj ĉiujn etikedojn kun prefiksoj de la sengepatra specimeno.

Lokaj kontroloj de ĉi tiu ŝanĝo:

```text
cargo fmt --all -- --check
(exit 0)
cargo clippy --workspace --release -- -D warnings
Finished `release` profile [optimized] target(s) in 4.62s
cargo test --workspace --all-features
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.17s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

La jam ekzistanta testo por 10 000 dosieroj estas ignorata nur en debug-reĝimo;
ĝia tempolimo validas por release. Neniu nova `allow` aŭ ignorata testo estis
aldonita. La nuna CI ne rulas la unuoptestojn; ĝi rulas formon, Clippy,
konstruadon, la `no_std`-sentineleton kaj la Kani-matricon.

**Nuba stato: ankoraŭ nekontrolita por la novaj pruvoj.** Nur post sukcesa
GitHub Actions-rulo de la ŝanĝita revizio (`Lint and build` kaj ĉiuj naŭ Kani-laboroj)
la formala akceptokriterio de Fazo 4 povas esti markita plenumita.
