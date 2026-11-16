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

/// Fiksa 10-bajta varinto por 128 en la malkodaj pruvoj.
const FIKSA_TEMPSTAMPO: [u8; 10] = [0x80, 0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00];
const FIKSA_KREITA_JE: u64 = 128;

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

/// La malseriigilo legas la saman kadron kun simbolikaj haketoj.
///
/// La mesaĝo kaj la tempstampo estas fiksaj, do la longoprefikso kaj la
/// varinto restas konataj al CBMC. La asertoj fiksas la tutan kadron escepte
/// de la du haketoj; ili estas la sama konstruo, kiun la pruvo faras, sed
/// malhelpas, ke CBMC traktu la longoprefikson kiel simbolon.
#[kani::proof]
#[kani::unwind(32)]
fn pruvo_komito_malseriigo() {
    let arbo: [u8; HAKETA_LONGO] = kani::any();
    let gepatra: [u8; HAKETA_LONGO] = kani::any();
    let havas_gepatron: bool = kani::any();

    kani::cover!(havas_gepatron);
    kani::cover!(!havas_gepatron);

    // La kadro estas konstruata permane; `testo_komito_fiksa_kodajo` kontrolas
    // la saman aranĝon.
    if havas_gepatron {
        let mut kadro = [0u8; 77];
        kadro[..HAKETA_LONGO].copy_from_slice(&arbo);
        kadro[HAKETA_LONGO] = 1;
        kadro[33..65].copy_from_slice(&gepatra);
        kadro[67..77].copy_from_slice(&FIKSA_TEMPSTAMPO);
        kadro[66] = b'a';
        // La longoprefikso estas la lasta skribo.
        kadro[65] = 1;

        kani::assume(kadro[..HAKETA_LONGO] == arbo[..]);
        kani::assume(kadro[33..65] == gepatra[..]);
        kani::assume(kadro[HAKETA_LONGO] == 1);
        kani::assume(kadro[65] == 1);
        kani::assume(kadro[66] == b'a');
        kani::assume(kadro[67..77] == FIKSA_TEMPSTAMPO[..]);

        let komito = Komito {
            arbo: Haketo::el_bajtoj(arbo),
            gepatro: Some(Haketo::el_bajtoj(gepatra)),
            mesagxo: String::from("a"),
            kreita_je: FIKSA_KREITA_JE,
        };
        kontroli_malseriigon(&kadro, &komito);
    } else {
        let mut kadro = [0u8; 45];
        kadro[..HAKETA_LONGO].copy_from_slice(&arbo);
        kadro[HAKETA_LONGO] = 0;
        kadro[35..45].copy_from_slice(&FIKSA_TEMPSTAMPO);
        kadro[34] = b'a';
        // La longoprefikso estas la lasta skribo.
        kadro[33] = 1;

        kani::assume(kadro[..HAKETA_LONGO] == arbo[..]);
        kani::assume(kadro[HAKETA_LONGO] == 0);
        kani::assume(kadro[33] == 1);
        kani::assume(kadro[34] == b'a');
        kani::assume(kadro[35..45] == FIKSA_TEMPSTAMPO[..]);

        let komito = Komito {
            arbo: Haketo::el_bajtoj(arbo),
            gepatro: None,
            mesagxo: String::from("a"),
            kreita_je: FIKSA_KREITA_JE,
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

/// Ĉiuj gepatro-etikedoj: 0 kaj 1 estas akceptataj, aliaj rifuzataj.
///
/// La kadroj estas fiksaj kaj la etikedo estas la sola simbolo. La asertoj
/// fiksas la ceterajn regajn bajtojn, por ke CBMC ne traktu la mesaĝolongon
/// aŭ la tempstampon kiel simbolojn.
#[kani::proof]
#[kani::unwind(16)]
fn pruvo_komito_nevalida_etikedo() {
    let etikedo: u8 = kani::any();

    let rezulto = if etikedo == 0 {
        let mut kadro = [0u8; 36];
        kadro[..HAKETA_LONGO].copy_from_slice(&[7u8; HAKETA_LONGO]);
        kadro[HAKETA_LONGO] = 0;
        kadro[33] = 0;
        kadro[34..36].copy_from_slice(&[0x80, 0x01]);

        kani::assume(kadro[HAKETA_LONGO] == 0);
        kani::assume(kadro[33] == 0);
        kani::assume(kadro[34] == 0x80);
        kani::assume(kadro[35] == 0x01);
        Komito::malseriigi(&kadro)
    } else {
        let mut kadro = [0u8; 68];
        kadro[..HAKETA_LONGO].copy_from_slice(&[7u8; HAKETA_LONGO]);
        kadro[HAKETA_LONGO] = etikedo;
        kadro[33..65].copy_from_slice(&[9u8; HAKETA_LONGO]);
        kadro[65] = 0;
        kadro[66..68].copy_from_slice(&[0x80, 0x01]);

        kani::assume(kadro[HAKETA_LONGO] == etikedo);
        kani::assume(kadro[65] == 0);
        kani::assume(kadro[66] == 0x80);
        kani::assume(kadro[67] == 0x01);
        Komito::malseriigi(&kadro)
    };

    let valida = etikedo <= 1;
    kani::cover!(etikedo == 0 && rezulto.is_ok());
    kani::cover!(etikedo == 1 && rezulto.is_ok());
    kani::cover!(etikedo > 1 && rezulto.is_err());
    assert!(rezulto.is_ok() == valida);
}

/// Ĉiuj propraj prefiksoj de fiksa sengepatra komito.
///
/// La buklo estas konkreta, do ĉiu tranĉo havas konkretan longon kaj Kani ne
/// devas trakti la longoprefikson kiel simbolon.
#[kani::proof]
#[kani::unwind(40)]
fn pruvo_komito_trancxo() {
    let mut kadro = [0u8; 37];
    kadro[..HAKETA_LONGO].copy_from_slice(&[7u8; HAKETA_LONGO]);
    kadro[HAKETA_LONGO..].copy_from_slice(&[0, 1, b'a', 0x80, 0x01]);

    for longo in 0..kadro.len() {
        let prefikso = &kadro[..longo];
        assert!(
            Komito::malseriigi(prefikso).is_err(),
            "ĉiu propra prefikso devas esti rifuzata"
        );
    }

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
