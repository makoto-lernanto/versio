//! Kani-pruvoj por la pura kerno de Versio.
//!
//! La pruvoj respektas la limojn de la konstitucio: malgrandaj enigoj kaj
//! `kani::cover!` por malhelpi malplenajn pruvojn.

use alloc::string::String;
use alloc::vec;

use crate::eraroj::Eraro;
use crate::kodigo::limigi_kvanton;
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

/// 1. Permuto de enskriboj ne ŝanĝas la seriigitajn bajtojn.
#[kani::proof]
#[kani::unwind(10)]
fn pruvo_arbo_kanonika() {
    let nomo_a = simbolika_nomo();
    let nomo_b = simbolika_nomo();
    // JUSTIFIKO: kanonika arbo malpermesas samajn nomojn; la pruvo temas pri
    // permuto de du malsamaj nomoj.
    kani::assume(nomo_a != nomo_b);

    let haketo = fiksa_haketo();
    let unua = Arbo::el_enskriboj(vec![
        ArbaEnskribo::Dosiero {
            nomo: String::from(nomo_a),
            haketo,
            grandeco: 0,
        },
        ArbaEnskribo::SubArbo {
            nomo: String::from(nomo_b),
            haketo,
        },
    ]);
    let dua = Arbo::el_enskriboj(vec![
        ArbaEnskribo::SubArbo {
            nomo: String::from(nomo_b),
            haketo,
        },
        ArbaEnskribo::Dosiero {
            nomo: String::from(nomo_a),
            haketo,
            grandeco: 0,
        },
    ]);

    kani::cover!(nomo_a < nomo_b);
    kani::cover!(nomo_a > nomo_b);
    assert!(
        unua.is_ok() && dua.is_ok(),
        "ambaŭ arboj devas esti validaj"
    );

    if let (Ok(unua), Ok(dua)) = (unua, dua) {
        let unua_bajtoj = unua.seriigi();
        let dua_bajtoj = dua.seriigi();
        assert!(
            unua_bajtoj.is_ok() && dua_bajtoj.is_ok(),
            "seriigado devas sukcesi"
        );
        if let (Ok(unua_bajtoj), Ok(dua_bajtoj)) = (unua_bajtoj, dua_bajtoj) {
            assert!(
                unua_bajtoj == dua_bajtoj,
                "permuto ne rajtas ŝanĝi la seriigitajn bajtojn"
            );
        }
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

/// 4. Ajna 16-bajta enigo al la limigita malseriigado redonas `Result`.
#[kani::proof]
#[kani::unwind(10)]
fn pruvo_dekodo_neniam_panikas() {
    let bajtoj: [u8; 16] = kani::any();
    let rezulto = Arbo::malseriigi(&bajtoj);

    kani::cover!(matches!(&rezulto, Err(Eraro::Malseriigo(_))));
    kani::cover!(matches!(&rezulto, Err(Eraro::NevalidaDatumo { .. })));

    // La sukcesa vojo estas atingebla per la valida kodigo de malplena arbo.
    assert!(
        Arbo::malseriigi(&[0]).is_ok(),
        "malplena arbo devas dekodiĝi"
    );
    assert!(
        Arbo::malseriigi(&[0, 0]).is_err(),
        "postrestaj bajtoj devas esti rifuzitaj"
    );
    assert!(
        Komito::malseriigi(&[]).is_err(),
        "malplena komito devas esti rifuzita"
    );
}

/// 5. Deklarita kvanto estas limigita antaŭ ol rezervi memoraĵon.
#[kani::proof]
#[kani::unwind(10)]
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

    // Malica trolonga longoprefikso estas rifuzata sen grandega rezervado.
    let malica = [0xffu8; 16];
    assert!(
        Arbo::malseriigi(&malica).is_err(),
        "trolonga prefikso devas esti rifuzita"
    );
}
