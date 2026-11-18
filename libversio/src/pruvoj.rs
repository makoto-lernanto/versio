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

/// Simbola, unu-bajta mesaĝo aŭ malplena mesaĝo.
fn prova_mesagxo(havas_mesagxon: bool, signo: u8) -> String {
    if havas_mesagxon {
        String::from(char::from(signo & 0x7f))
    } else {
        String::new()
    }
}

/// Kontrolas la sukceson, la longan intervalon kaj la arban prefikson.
fn kontroli_kadron(komito: &Komito, arbo: &[u8; HAKETA_LONGO], minimumo: usize, maksimumo: usize) {
    // JUSTIFIKO: `Eraro` enhavas `Vec<String>` por la kunfandaj konfliktoj, kaj
    // ties forĵeta gluo estas multekosta por la solvilo kvankam ĝi ne apartenas
    // al la teoremo. `ManuallyDrop` evitigas tiun gluon; la aserto pri sukceso
    // kaj la kadraj limoj restas plene kontrolataj.
    let kodita = core::mem::ManuallyDrop::new(komito.seriigi());
    assert!(kodita.is_ok());
    if let Ok(bajtoj) = &*kodita {
        assert!(bajtoj.len() >= minimumo);
        assert!(bajtoj.len() <= maksimumo);
        assert!(bajtoj.get(..HAKETA_LONGO) == Some(&arbo[..]));
    }
}

/// Seriigo de la radika komito (sen gepatro).
///
/// Ĉiu kadra formo havas propran pruvon kun fiksaj gepatroj: se la gepatroj
/// estus simbolaj buleanoj, la solvilo esplorus ĉiujn formojn en unu formulo.
/// Kune la tri pruvoj kovras la saman enigan spacon kiel unu sola voko.
///
/// La malkunigo de 40 estas necesa: la haketo estas 32 bajtoj, do la
/// `memcpy`- kaj `memcmp`-bukloj de la norma biblioteko bezonas pli ol 32
/// iteraciojn por esti plene modelitaj. Pli malgranda valoro tranĉas la
/// komparadon kaj la kopiadon duone, kio kaŝus erarojn en la postaj bajtoj.
#[kani::proof]
#[kani::unwind(40)]
fn pruvo_komito_seriigo() {
    let arbo: [u8; HAKETA_LONGO] = kani::any();
    let havas_mesagxon: bool = kani::any();
    let signo: u8 = kani::any();
    let kreita_je: u64 = kani::any();
    let komito = Komito {
        arbo: Haketo::el_bajtoj(arbo),
        gepatro: None,
        mesagxo: prova_mesagxo(havas_mesagxon, signo),
        kreita_je,
        dua_gepatro: None,
    };

    kani::cover!(havas_mesagxon);
    kani::cover!(!havas_mesagxon);
    kani::cover!(kreita_je == u64::MAX);
    kani::cover!(kreita_je == 0);
    kontroli_kadron(&komito, &arbo, 36, 46);
}

/// Seriigo de ordinara komito kun unu gepatro.
#[kani::proof]
#[kani::unwind(40)]
fn pruvo_gepatra_kadro() {
    let arbo: [u8; HAKETA_LONGO] = kani::any();
    let gepatra: [u8; HAKETA_LONGO] = kani::any();
    let havas_mesagxon: bool = kani::any();
    let signo: u8 = kani::any();
    let kreita_je: u64 = kani::any();
    let komito = Komito {
        arbo: Haketo::el_bajtoj(arbo),
        gepatro: Some(Haketo::el_bajtoj(gepatra)),
        mesagxo: prova_mesagxo(havas_mesagxon, signo),
        kreita_je,
        dua_gepatro: None,
    };

    kani::cover!(havas_mesagxon);
    kani::cover!(!havas_mesagxon);
    kani::cover!(kreita_je == u64::MAX);
    kani::cover!(kreita_je == 0);
    kontroli_kadron(&komito, &arbo, 68, 78);
}

/// Seriigo de kunfanda komito kun du gepatroj.
#[kani::proof]
#[kani::unwind(40)]
fn pruvo_kunfanda_kadro() {
    let arbo: [u8; HAKETA_LONGO] = kani::any();
    let unua: [u8; HAKETA_LONGO] = kani::any();
    let dua: [u8; HAKETA_LONGO] = kani::any();
    let havas_mesagxon: bool = kani::any();
    let signo: u8 = kani::any();
    let kreita_je: u64 = kani::any();
    let komito = Komito {
        arbo: Haketo::el_bajtoj(arbo),
        gepatro: Some(Haketo::el_bajtoj(unua)),
        mesagxo: prova_mesagxo(havas_mesagxon, signo),
        kreita_je,
        dua_gepatro: Some(Haketo::el_bajtoj(dua)),
    };

    kani::cover!(havas_mesagxon);
    kani::cover!(!havas_mesagxon);
    kani::cover!(kreita_je == u64::MAX);
    kani::cover!(kreita_je == 0);
    kontroli_kadron(&komito, &arbo, 100, 110);
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
