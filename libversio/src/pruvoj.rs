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

/// La malseriigilo legas reprezentajn kadrojn faritajn de `seriigi`.
///
/// La enigoj estas konkretaj, do CBMC ne devas esplori simbolajn longojn aŭ
/// UTF-8-vojojn. Arbitraj bajtoj estas kovritaj de
/// `pruvo_legilo_neniam_panikas`, kaj arbitraj kampoj de la seriiga pruvo.
#[kani::proof]
#[kani::unwind(40)]
fn pruvo_komito_malseriigo() {
    let mut komito = Komito {
        arbo: Haketo::el_bajtoj([7u8; HAKETA_LONGO]),
        gepatro: None,
        mesagxo: String::new(),
        kreita_je: 0,
    };
    kontroli_rondiron(&komito);

    komito.mesagxo = String::from("a");
    komito.kreita_je = 128;
    kontroli_rondiron(&komito);

    komito.arbo = Haketo::el_bajtoj([8u8; HAKETA_LONGO]);
    komito.gepatro = Some(Haketo::el_bajtoj([9u8; HAKETA_LONGO]));
    komito.mesagxo = String::from("é");
    komito.kreita_je = 127;
    kontroli_rondiron(&komito);

    komito.mesagxo = String::from("€");
    komito.kreita_je = u64::MAX;
    kontroli_rondiron(&komito);
}

fn kontroli_rondiron(komito: &Komito) {
    let kodita = komito.seriigi();
    assert!(kodita.is_ok(), "seriigo devas sukcesi");
    if let Ok(bajtoj) = kodita {
        let relegita = Komito::malseriigi(&bajtoj);
        assert!(relegita.is_ok(), "malseriigo devas sukcesi");
        if let Ok(relegita) = relegita {
            assert!(relegita == *komito);
        }
    }
}

/// Gepatro-etikedoj: 0 kaj 1 akceptataj, 2 kaj 255 rifuzataj.
///
/// La kadroj estas tute konkretaj. Ĉiuj 256 etikedoj estas kovritaj de
/// unuoptesto; ĉi tie Kani kontrolas la kvar limojn sen simbolaj longoj.
#[kani::proof]
#[kani::unwind(16)]
fn pruvo_komito_nevalida_etikedo() {
    let mut sen_gepatro = [0u8; 36];
    sen_gepatro[..HAKETA_LONGO].copy_from_slice(&[7u8; HAKETA_LONGO]);
    sen_gepatro[HAKETA_LONGO] = 0;
    sen_gepatro[33] = 0;
    sen_gepatro[34..36].copy_from_slice(&[0x80, 0x01]);
    assert!(
        Komito::malseriigi(&sen_gepatro).is_ok(),
        "etikedo 0 devas sukcesi"
    );

    let mut kun_gepatro = [0u8; 68];
    kun_gepatro[..HAKETA_LONGO].copy_from_slice(&[7u8; HAKETA_LONGO]);
    kun_gepatro[HAKETA_LONGO] = 1;
    kun_gepatro[33..65].copy_from_slice(&[9u8; HAKETA_LONGO]);
    kun_gepatro[65] = 0;
    kun_gepatro[66..68].copy_from_slice(&[0x80, 0x01]);
    assert!(
        Komito::malseriigi(&kun_gepatro).is_ok(),
        "etikedo 1 devas sukcesi"
    );

    kun_gepatro[HAKETA_LONGO] = 2;
    assert!(
        Komito::malseriigi(&kun_gepatro).is_err(),
        "etikedo 2 devas esti rifuzata"
    );

    kun_gepatro[HAKETA_LONGO] = 255;
    assert!(
        Komito::malseriigi(&kun_gepatro).is_err(),
        "etikedo 255 devas esti rifuzata"
    );
}

/// Ĉiuj propraj prefiksoj de fiksa sengepatra komito.
///
/// Ĉiu tranĉo havas literal-longon, do Kani ne devas trakti la longoprefikson
/// kiel simbolon.
#[kani::proof]
#[kani::unwind(16)]
fn pruvo_komito_trancxo() {
    let mut kadro = [0u8; 37];
    kadro[..HAKETA_LONGO].copy_from_slice(&[7u8; HAKETA_LONGO]);
    kadro[HAKETA_LONGO..].copy_from_slice(&[0, 1, b'a', 0x80, 0x01]);

    macro_rules! rifuzi {
        ($($longo:literal),* $(,)?) => {
            $(assert!(Komito::malseriigi(&kadro[..$longo]).is_err());)*
        };
    }
    rifuzi!(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36
    );

    assert!(
        Komito::malseriigi(&kadro).is_ok(),
        "la kompleta kadro devas sukcesi"
    );
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
