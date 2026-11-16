//! Kani-pruvoj por la pura kerno de Versio.
//!
//! La pruvoj respektas la limojn de la konstitucio: malgrandaj enigoj kaj
//! `kani::cover!` por malhelpi malplenajn pruvojn.

use alloc::string::String;
use alloc::vec;

use crate::eraroj::Eraro;
use crate::kodigo::{Legilo, limigi_kvanton};
use crate::objektoj::{ArbaEnskribo, Arbo, Komito, MAKSIMAJ_ENSKRIBOJ};
use crate::stokado::{HAKETA_LONGO, Haketo};
use crate::vojo::validi_bajtojn;

fn simbolika_nomo() -> char {
    let bajto: u8 = kani::any();
    // JUSTIFIKO: nur minusklaj ASCII-literoj estas akceptataj kiel unu-signaj
    // nomoj en la pruvoj; aliaj bajtoj ne apartenas al la modelo.
    kani::assume(bajto.is_ascii_lowercase());
    char::from(bajto)
}

fn fiksa_haketo() -> Haketo {
    Haketo::el_bajtoj([7u8; HAKETA_LONGO])
}

/// 1. La kanonika ordo estas sendependa de la eniga ordo.
///
/// Unu voko de `el_enskriboj` sufiĉas: la eniga ordo mem estas simbolo, do
/// ambaŭ permutoj estas esplorataj. Dua aparta voko duobligas la formulon kaj
/// superŝutas la solvilon en la nuba rulilo.
#[kani::proof]
#[kani::unwind(4)]
fn pruvo_arbo_kanonika() {
    let nomo_a = simbolika_nomo();
    let nomo_b = simbolika_nomo();
    // JUSTIFIKO: kanonika arbo malpermesas samajn nomojn; la pruvo temas pri
    // permuto de du malsamaj nomoj.
    kani::assume(nomo_a != nomo_b);

    let haketo = fiksa_haketo();
    let dosiero = ArbaEnskribo::Dosiero {
        nomo: String::from(nomo_a),
        haketo,
        grandeco: 0,
    };
    let subarbo = ArbaEnskribo::SubArbo {
        nomo: String::from(nomo_b),
        haketo,
    };

    let ordo: bool = kani::any();
    kani::cover!(ordo);
    kani::cover!(!ordo);

    let arbo = Arbo::el_enskriboj(if ordo {
        vec![dosiero, subarbo]
    } else {
        vec![subarbo, dosiero]
    });

    assert!(arbo.is_ok(), "la arbo devas esti valida");
    if let Ok(arbo) = arbo {
        let enskriboj = arbo.enskriboj();
        assert!(enskriboj.len() == 2, "la arbo devas havi du enskribojn");
        assert!(
            enskriboj[0].nomo() < enskriboj[1].nomo(),
            "la enskriboj devas esti kanonike ordigitaj"
        );
        assert!(
            matches!(
                (&enskriboj[0], &enskriboj[1]),
                (ArbaEnskribo::Dosiero { .. }, ArbaEnskribo::SubArbo { .. })
                    | (ArbaEnskribo::SubArbo { .. }, ArbaEnskribo::Dosiero { .. })
            ),
            "ambaŭ enskriboj devas resti en la arbo"
        );
    }
}

/// 2. Duplikataj vojoj estas rifuzitaj.
#[kani::proof]
#[kani::unwind(10)]
fn pruvo_arbo_duplikatoj() {
    let nomo = simbolika_nomo();
    let haketo = fiksa_haketo();

    let rezulto = Arbo::el_enskriboj(vec![
        ArbaEnskribo::Dosiero {
            nomo: String::from(nomo),
            haketo,
            grandeco: 0,
        },
        ArbaEnskribo::SubArbo {
            nomo: String::from(nomo),
            haketo,
        },
    ]);

    kani::cover!(matches!(&rezulto, Err(Eraro::DuplikataVojo { .. })));
    assert!(
        matches!(&rezulto, Err(Eraro::DuplikataVojo { .. })),
        "duplikata nomo devas esti rifuzita"
    );

    // Pozitiva kontrolo: malsamaj nomoj estas akceptataj.
    let valida = Arbo::el_enskriboj(vec![
        ArbaEnskribo::Dosiero {
            nomo: String::from("a"),
            haketo,
            grandeco: 0,
        },
        ArbaEnskribo::SubArbo {
            nomo: String::from("b"),
            haketo,
        },
    ]);
    assert!(valida.is_ok(), "malsamaj nomoj devas esti akceptitaj");
}

/// 3. Danĝeraj vojkonsisteroj estas rifuzitaj; ajna enigo redonas `Result`.
#[kani::proof]
#[kani::unwind(10)]
fn pruvo_vojo_sekura() {
    let bajtoj: [u8; 8] = kani::any();
    let rezulto = validi_bajtojn(&bajtoj);

    kani::cover!(rezulto.is_ok());
    kani::cover!(rezulto.is_err());
    if rezulto.is_ok() {
        for &bajto in &bajtoj {
            assert!(bajto != b'/', "oblikva streko devas esti rifuzita");
            assert!(bajto != b'\\', "kontraŭoblikva streko devas esti rifuzita");
            assert!(bajto != 0, "NUL-signo devas esti rifuzita");
        }
    }

    assert!(
        validi_bajtojn(b"").is_err(),
        "malplena nomo devas esti rifuzita"
    );
    assert!(validi_bajtojn(b"..").is_err(), "«..» devas esti rifuzita");
    assert!(validi_bajtojn(b"a/b").is_err(), "«/» devas esti rifuzita");
}

/// 4. La legilo neniam panikas sur nekonfidindaj bajtoj.
///
/// La plena malkodado de arboj kaj komitoj estas kovrita de unuoptestoj; ĉi
/// tie Kani kontrolas la varintojn kaj la limojn, kiuj estas la sola parto
/// de la malkodado, kiu ne delegas al la norma biblioteko.
#[kani::proof]
#[kani::unwind(12)]
fn pruvo_legilo_neniam_panikas() {
    let bajtoj: [u8; 16] = kani::any();
    let longo: usize = kani::any();

    let mut unua = Legilo::nova(&bajtoj);
    kani::cover!(unua.bajtoj(longo).is_ok());
    let mut dua = Legilo::nova(&bajtoj);
    kani::cover!(dua.bajtoj(longo).is_err());

    let mut legilo = Legilo::nova(&bajtoj);
    let _ = legilo.u8();
    let _ = legilo.u32();
    let _ = legilo.u64();
    let _ = legilo.haketo();
    let _ = legilo.fino();
}

/// La seriigilo akceptas simbolikajn kampojn kaj respektas la kadran longon.
#[kani::proof]
#[kani::unwind(40)]
fn pruvo_komito_seriigo() {
    let arbo: [u8; HAKETA_LONGO] = kani::any();
    let gepatra: [u8; HAKETA_LONGO] = kani::any();
    let havas_gepatron: bool = kani::any();
    let havas_mesagxon: bool = kani::any();
    let signo: u8 = kani::any();
    let kreita_je: u64 = kani::any();
    let komito = Komito {
        arbo: Haketo::el_bajtoj(arbo),
        gepatro: havas_gepatron.then_some(Haketo::el_bajtoj(gepatra)),
        mesagxo: if havas_mesagxon {
            String::from(char::from(signo & 0x7f))
        } else {
            String::new()
        },
        kreita_je,
    };

    kani::cover!(havas_gepatron && havas_mesagxon);
    kani::cover!(!havas_gepatron && !havas_mesagxon);
    kani::cover!(kreita_je == u64::MAX);
    kani::cover!(kreita_je == 0);
    let kodita = komito.seriigi();
    assert!(kodita.is_ok());
    if let Ok(bajtoj) = kodita {
        assert!(bajtoj.len() >= 35);
        assert!(bajtoj.len() <= 77);
        assert!(bajtoj.get(..HAKETA_LONGO) == Some(&arbo[..]));
    }
}

/// La malseriigilo legas la saman kadron kun simbolikaj enhavoj.
#[kani::proof]
#[kani::unwind(32)]
fn pruvo_komito_malseriigo() {
    let arbo: [u8; HAKETA_LONGO] = kani::any();
    let gepatra: [u8; HAKETA_LONGO] = kani::any();
    let havas_gepatron: bool = kani::any();
    let signo: u8 = kani::any();
    let kreita_je: u64 = kani::any();

    kani::cover!(havas_gepatron);
    kani::cover!(!havas_gepatron);
    kani::cover!(kreita_je == u64::MAX);

    // La kadro estas konstruata permane por ke la longoprefikso restu
    // konkreta; `testo_komito_fiksa_kodajo` kontrolas la saman aranĝon.
    //
    // JUSTIFIKO: la kadro estas fiksa staka tabelo, ne `Vec`. Kani traktas
    // `Vec`-enhavon kun simbolaj kampoj kiel unu simbolan tabelon, do ankaŭ la
    // konkreta mesaĝolongo fariĝus simbolo kaj la UTF-8-validigo eksplodus.
    // La tempstampo estas skribata per valida, fiksa 10-bajta varinto.
    if havas_gepatron {
        let mut kadro = [0u8; 77];
        kadro[..HAKETA_LONGO].copy_from_slice(&arbo);
        kadro[HAKETA_LONGO] = 1;
        kadro[33..65].copy_from_slice(&gepatra);
        let mut tempstampo = [0u8; 10];
        skribi_tempstampon(kreita_je, &mut tempstampo);
        kadro[67..77].copy_from_slice(&tempstampo);
        kadro[66] = signo & 0x7f;
        // La longoprefikso estas la lasta skribo, do Kani ne povas konfuzi
        // ĝin kun la simbolaj haketoj aŭ tempstampo.
        kadro[65] = 1;
        let komito = Komito {
            arbo: Haketo::el_bajtoj(arbo),
            gepatro: Some(Haketo::el_bajtoj(gepatra)),
            mesagxo: String::from(char::from(signo & 0x7f)),
            kreita_je,
        };
        kontroli_malseriigon(&kadro, &komito);
    } else {
        let mut kadro = [0u8; 45];
        kadro[..HAKETA_LONGO].copy_from_slice(&arbo);
        kadro[HAKETA_LONGO] = 0;
        let mut tempstampo = [0u8; 10];
        skribi_tempstampon(kreita_je, &mut tempstampo);
        kadro[35..45].copy_from_slice(&tempstampo);
        kadro[34] = signo & 0x7f;
        // La longoprefikso estas la lasta skribo.
        kadro[33] = 1;
        let komito = Komito {
            arbo: Haketo::el_bajtoj(arbo),
            gepatro: None,
            mesagxo: String::from(char::from(signo & 0x7f)),
            kreita_je,
        };
        kontroli_malseriigon(&kadro, &komito);
    }
}

fn kontroli_malseriigon(bajtoj: &[u8], komito: &Komito) {
    let rezulto = Komito::malseriigi(bajtoj);
    assert!(rezulto.is_ok());
    if let Ok(relegita) = rezulto {
        assert!(relegita == *komito);
    }
}

/// Skribas ajnan `u64` kiel validan fiksan 10-bajtan varinton.
///
/// La kutima `postcard`-kodigo estas pli mallonga por malgrandaj valoroj; la
/// fiksa longo tenas la kadron kaj ties mesaĝolongon konkretaj por Kani.
fn skribi_tempstampon(valoro: u64, celo: &mut [u8; 10]) {
    for (indekso, bajto) in celo.iter_mut().enumerate().take(9) {
        *bajto = (valoro >> (7 * indekso)) as u8 & 0x7f | 0x80;
    }
    celo[9] = (valoro >> 63) as u8 & 0x01;
}

/// Ĉiuj gepatro-etikedoj: 0 kaj 1 estas akceptataj, aliaj rifuzataj.
///
/// La kadroj estas tute konkretaj krom la etikedo, do la pruvo restas malpeza
/// kaj sendependa de la mesaĝa malkodado.
#[kani::proof]
#[kani::unwind(16)]
fn pruvo_komito_nevalida_etikedo() {
    let etikedo: u8 = kani::any();

    // Por etikedo 0 la kadro estas sengepatra; por 1 (kaj por ĉiu alia valoro,
    // kiun la sintakso ankoraŭ legas) la kadro entenas plenan gepatron.
    let rezulto = if etikedo == 0 {
        let mut kadro = [0u8; 36];
        kadro[..HAKETA_LONGO].copy_from_slice(&[7u8; HAKETA_LONGO]);
        kadro[HAKETA_LONGO] = 0;
        kadro[33] = 0;
        kadro[34..36].copy_from_slice(&[0x80, 0x01]);
        Komito::malseriigi(&kadro)
    } else {
        let mut kadro = [0u8; 68];
        kadro[..HAKETA_LONGO].copy_from_slice(&[7u8; HAKETA_LONGO]);
        kadro[HAKETA_LONGO] = etikedo;
        kadro[33..65].copy_from_slice(&[9u8; HAKETA_LONGO]);
        kadro[65] = 0;
        kadro[66..68].copy_from_slice(&[0x80, 0x01]);
        Komito::malseriigi(&kadro)
    };

    let valida = etikedo <= 1;
    kani::cover!(etikedo == 0 && rezulto.is_ok());
    kani::cover!(etikedo == 1 && rezulto.is_ok());
    kani::cover!(etikedo > 1 && rezulto.is_err());
    assert!(rezulto.is_ok() == valida);
}

/// Ĉiuj prefiksoj de fiksa sengepatra komito.
#[kani::proof]
#[kani::unwind(16)]
fn pruvo_komito_trancxo() {
    let mut kadro = [0u8; 37];
    kadro[..HAKETA_LONGO].copy_from_slice(&[7u8; HAKETA_LONGO]);
    kadro[HAKETA_LONGO..].copy_from_slice(&[0, 1, b'a', 0x80, 0x01]);
    let longo = usize::from(kani::any::<u8>());
    if let Some(prefikso) = kadro.get(..longo) {
        let rezulto = Komito::malseriigi(prefikso);
        let valida = longo == kadro.len();
        kani::cover!(valida && rezulto.is_ok());
        kani::cover!(longo == 0 && rezulto.is_err());
        kani::cover!(longo == HAKETA_LONGO && rezulto.is_err());
        assert!(rezulto.is_ok() == valida);
    }
}

/// 5. Deklarita kvanto estas limigita antaŭ ol rezervi memoraĵon.
#[kani::proof]
#[kani::unwind(4)]
fn pruvo_dekodo_limigita() {
    let kvanto: u32 = kani::any();
    let rezulto = limigi_kvanton(kvanto, 16, MAKSIMAJ_ENSKRIBOJ);

    kani::cover!(rezulto.is_ok());
    kani::cover!(rezulto.is_err());
    if let Ok(valoro) = rezulto {
        assert!(
            valoro <= MAKSIMAJ_ENSKRIBOJ,
            "la supra limo devas esti respektata"
        );
        assert!(
            valoro <= 16 / 35,
            "la kvanto devas kongrui kun la restantaj bajtoj"
        );
    }
}
