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
    // JUSTIFIKO: la malkodado okazas ene de ĉiu branĉo. Se la kadro estus
    // konstruata antaŭ la branĉo, CBMC kunfandus du malsamajn bufrojn kaj
    // traktus la konkretan mesaĝolongon kiel simbolan valoron, kio eksplodigus
    // la UTF-8-validigon.
    if havas_gepatron {
        let mut bajtoj = alloc::vec::Vec::new();
        bajtoj.extend_from_slice(&arbo);
        bajtoj.push(1);
        bajtoj.extend_from_slice(&gepatra);
        bajtoj.extend_from_slice(&[1, signo & 0x7f]);
        let tempstampo = postcard::to_allocvec(&kreita_je);
        assert!(tempstampo.is_ok());
        if let Ok(tempstampo) = tempstampo {
            bajtoj.extend_from_slice(&tempstampo);
            let komito = Komito {
                arbo: Haketo::el_bajtoj(arbo),
                gepatro: Some(Haketo::el_bajtoj(gepatra)),
                mesagxo: String::from(char::from(signo & 0x7f)),
                kreita_je,
            };
            kontroli_malseriigon(&bajtoj, &komito);
        }
    } else {
        let mut bajtoj = alloc::vec::Vec::new();
        bajtoj.extend_from_slice(&arbo);
        bajtoj.push(0);
        bajtoj.extend_from_slice(&[1, signo & 0x7f]);
        let tempstampo = postcard::to_allocvec(&kreita_je);
        assert!(tempstampo.is_ok());
        if let Ok(tempstampo) = tempstampo {
            bajtoj.extend_from_slice(&tempstampo);
            let komito = Komito {
                arbo: Haketo::el_bajtoj(arbo),
                gepatro: None,
                mesagxo: String::from(char::from(signo & 0x7f)),
                kreita_je,
            };
            kontroli_malseriigon(&bajtoj, &komito);
        }
    }
}

fn kontroli_malseriigon(bajtoj: &[u8], komito: &Komito) {
    let rezulto = Komito::malseriigi(bajtoj);
    assert!(rezulto.is_ok());
    if let Ok(relegita) = rezulto {
        assert!(relegita == *komito);
    }
}

/// Ĉiuj prefiksoj kaj gepatro-etikedoj de fiksa sengepatra komito.
#[kani::proof]
#[kani::unwind(16)]
fn pruvo_komito_nevalida_kodigo() {
    let mut bajtoj = alloc::vec::Vec::new();
    bajtoj.extend_from_slice(&[7u8; HAKETA_LONGO]);
    bajtoj.extend_from_slice(&[0, 1, b'a', 0x80, 0x01]);
    let etikedo: u8 = kani::any();
    bajtoj[HAKETA_LONGO] = etikedo;
    let longo = usize::from(kani::any::<u8>());
    if let Some(prefikso) = bajtoj.get(..longo) {
        let rezulto = Komito::malseriigi(prefikso);
        let valida = longo == bajtoj.len() && etikedo == 0;
        kani::cover!(valida && rezulto.is_ok());
        kani::cover!(longo == 0 && rezulto.is_err());
        kani::cover!(longo == bajtoj.len() && etikedo == 1 && rezulto.is_err());
        kani::cover!(longo == bajtoj.len() && etikedo > 1 && rezulto.is_err());
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
