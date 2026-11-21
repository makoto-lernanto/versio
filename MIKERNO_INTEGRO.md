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

`vs ini` povas esti rulata ankaŭ en la origina Git-deponejo: nun Versio
preterlasas `.git`, `target` kaj `.vs` kaj interpretas `.vsignore`. Ĝi tamen ne
interpretas `.gitignore`; por ekskludi pliajn dosierojn, uzu `.vsignore`.
Ĝi konservas ordinarajn dosierojn kaj arbojn, ne plenumeblajn permesbitojn aŭ
simbolajn ligilojn.

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
    "$VS" ini
    "$VS" stato
    "$VS" komitu --noto "Importi MiKerno $REV"
    "$VS" stato
)
printf 'Git-komito: %s\nVersio-deponejo: %s\n' "$REV" "$CELO"
```

La unua `stato` listigu la eksportitajn dosierojn sub `Novaj:`; la dua
redonu `Neniu ŝanĝo`. `komitu` kaptas la tutan labordosierujon: aparta
`aldoni`-komando ne ekzistas. La nova deponejo restas en `$CELO` por inspektado.

Por la ĉi-suba testita momentfoto, daŭrigu en la sama ŝelo. La ŝanĝoj tuŝas
nur la provizoran kopion; la forigita dosiero restas en la origina Git-komito.

```bash
(
    cd "$CELO"
    printf '\nVersio-integra provo.\n' >> README.md
    printf 'Nova dosiero por la integra provo.\n' > integra-provo.txt
    rm -- docs/VORTARO.md
    "$VS" stato
    "$VS" komitu --noto "Kontroli modifon aldonon kaj forigon"
    "$VS" stato
)
```

## Registrita loka provo

Dato: 2026-11-16. Medio: Linux/WSL2, x86_64; loka `target/debug/vs`
rekonstruita per la unuoptestoj. La KI-logiko estas tiu de Versio `68be503`;
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

La du Versio-komitoj havis malsamajn identigilojn. La testprogramo konfirmis, ke
la fonto restis netuŝita kaj la provo sukcesis. Komitaj identigiloj dependas ankaŭ
de tempstampo, do alia rulo ne devas reprodukti tiujn samajn identigilojn.

## Formala pruvado kaj ĝiaj limoj

La kvin antaŭaj Kani-pruvoj restas sen ŝanĝo de sia amplekso. Aldoniĝas tri
sendependaj DI-laboroj por la tri kadraj formoj de la komita seriigado. La
plena komita malkodado restis ekster la formala modelo pro mezurita limo de
CBMC, priskribita sube.

| Pruvo | Preciza modelo kaj aserto |
| --- | --- |
| `pruvo_komito_seriigo` | Radika komito sen gepatroj: simbolika radika haketo, simbolika ĉeesto de unu-bajta mesaĝo (la enhavo estas fiksa) kaj ajna `u64`-tempstampo. Seriigo devas sukcesi, la eliglongo resti inter 36 kaj 46 bajtoj, kaj la unuaj 32 bajtoj egali la radikan haketon. |
| `pruvo_gepatra_kadro` | Unu-gepatra komito: simbolika radika haketo, simbolika ĉeesto de mesaĝo kaj ajna `u64`-tempstampo; la gepatra haketo estas fiksa, ĉar neniu aserto komparas ĝiajn bajtojn. La eliglongo devas resti inter 68 kaj 78 bajtoj, kun la radika haketo unue. |
| `pruvo_kunfanda_kadro` | Kunfanda komito kun du gepatroj: simbolika radika haketo, simbolika ĉeesto de mesaĝo kaj ajna `u64`-tempstampo; ambaŭ gepatraj haketoj estas fiksaj. La eliglongo devas resti inter 100 kaj 110 bajtoj, kun la radika haketo unue. |

La kadro skribas la fiksajn kampojn (haketoj) antaŭ la varilongajn (mesaĝo,
tempstampo). Tial ambaŭ patraj haketoj sidas ĉe konstantaj pozicioj kaj la
variaĵoj restas malgrandaj: la kunfanda pruvo generas ĉirkaŭ 1,4 milionojn da
klaŭzoj; kiam la dua patra haketo sekvis la varintan tempstampon, la sama
pruvo generis 77 milionojn kaj estis nuligata de la ekzekutilo.

Oni provis tri pliajn pruvojn pri `Komito::malseriigi`: rondiron de reprezentaj
komitoj, ĉiujn gepatro-etikedojn kaj rifuzon de tranĉitaj kadroj. Eĉ kun tute
konkretaj enigaj kadroj la postcard-malkodado kaj `String::from_utf8` generis pli
ol 24 000 VCC kaj pli ol 300 sekundojn da simbola plenumo; la laboro restis
senfina kaj estis nuligita en DI. CBMC ankaŭ ne povas simboligi nur la
kontrolbajtojn de staka tabelo: se unu bajto estas simbola, la tuta tabelo
fariĝas simbola. Plilongigi la tempolimon ne solvis tion, kaj `kani::assume` por
kaŝi la malfermajn vojojn estus malpermesita malfortigo. Tiuj tri pruvoj do
estis forigitaj; la malkodado restas kovrita de la unuoptestoj kaj de
`pruvo_legilo_neniam_panikas`, kiu legas arbitrajn bajtojn sen paniki.

La malvolva limo de la tri kadraj pruvoj estas 40; la malvolvaj asertoj ne
estas malŝaltitaj. Ĉiu el ili havas la saman 600-sekundan komandlimon kaj
30-minutan laborlimon kiel la aliaj. Neniu pruvo estas rulata loke.

Tio estas limigita modelo, ne pruvo por ĉiu arbitra komito, ĉiu UTF-8-mesaĝo,
ĉiu arbo aŭ por la kriptografia sekureco de BLAKE3. Arbitraj mesaĝoj kaj
tempstampoj estas kovritaj de unuoptestoj, ne de Kani. Tiuj testoj kovras ambaŭ
gepatro-kazojn, malplenajn/Unikodajn/maksimumlongajn mesaĝojn, varintajn
tempstampolimojn, ĉiujn verajn prefiksojn de du specimenaj komitoj,
postrestajn bajtojn, ĉiujn gepatro-etikedojn kaj ĉiujn etikedojn kun prefiksoj de
la sengepatra specimeno.

Lokaj kontroloj de ĉi tiu ŝanĝo:

```text
cargo fmt --all -- --check
(exit 0)
cargo clippy --workspace --release -- -D warnings
Finished `release` profile [optimized] target(s) in 1.21s
cargo test --workspace --all-features
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

La ceteraj testaroj de la laborloko ankaŭ estis verdaj (3, 5, 4, 4, 6 kaj 8
testoj).

La jam ekzistanta testo por 10 000 dosieroj estas ignorata nur en la sencimiga
profilo; ĝia tempolimo validas por la eldona profilo. Neniu nova `allow` aŭ
ignorata testo estis aldonita. La nuna DI ne rulas la unuoptestojn; ĝi rulas
formon, Clippy, konstruadon, la `no_std`-sentineleton kaj la Kani-matricon.

**Nuba stato: verda.** GitHub Actions-rulo
[`44302583687`](https://github.com/makoto-lernanto/versio/actions/runs/44302583687)
de revizio `580dfb8` sukcese finis `Kontroli kaj konstrui` kaj ĉiujn ok
`Kani (...)`-laborojn (2026-11-18). La plej longa pruvo,
`pruvo_arbo_kanonika`, daŭris 4 minutojn kaj 47 sekundojn; la kunfanda kadro
daŭris 1 minuton kaj 41 sekundojn, kaj ĉiuj restis sub la 600-sekunda limo.
Tiel la formala akceptokriterio de Fazo 4 estas plenumita.
