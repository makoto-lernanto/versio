//! Bare-metal memtesto de la pura kerno sur aarch64.
//!
//! Ĝi estas konstruata por `aarch64-unknown-none` kaj rulata per
//! `qemu-system-aarch64 -M virt -semihosting`. La eligo iras tra
//! semihosting, do ne ekzistas gastiga operaciumo aŭ `std`.
//!
//! Ĉiu kontrolo uzas la saman publikan interfacon kiel la hostaj testoj; se la
//! binaro eliras kun 0, ĉiuj kontroloj sukcesis sur reala ARM64-paperto.

#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use libversio::{
    ArbaEnskribo, Arbo, Eraro, Haketo, Konservilo, MemoraKonservilo, haketi, validi_bajtojn,
};

/// La oficiala `BLAKE3`-vektoro por la malplena enigo.
const BLAKE3_MALPLENA: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

/// Etaj statistikoj por la raporto.
struct Rezultoj {
    sukcesoj: u32,
    malsukcesoj: u32,
}

impl Rezultoj {
    const fn nova() -> Self {
        Self {
            sukcesoj: 0,
            malsukcesoj: 0,
        }
    }

    fn kontroli(&mut self, nomo: &str, kondiĉo: bool) {
        if kondiĉo {
            self.sukcesoj = self.sukcesoj.saturating_add(1);
            semihosting::println!("  OK   {nomo}");
        } else {
            self.malsukcesoj = self.malsukcesoj.saturating_add(1);
            semihosting::println!("  FAIL {nomo}");
        }
    }
}

/// Elirpunkto post la starigo de la stako fare de `_start`.
fn rust_main() -> ! {
    let mut r = Rezultoj::nova();

    semihosting::println!("Versio arm64-memtesto");

    kontroli_haketilon(&mut r);
    kontroli_konservilon(&mut r);
    kontroli_arbon(&mut r);
    kontroli_vojon(&mut r);
    kontroli_dekodon(&mut r);

    r.kontroli("usize estas 64-bita", core::mem::size_of::<usize>() == 8);
    r.kontroli(
        "la celo estas little-endian",
        cfg!(target_endian = "little"),
    );

    semihosting::println!(
        "arm64: {} sukcesoj, {} malsukcesoj",
        r.sukcesoj,
        r.malsukcesoj
    );
    semihosting::process::exit(i32::from(r.malsukcesoj != 0))
}

/// La reala (pure-Rust) `BLAKE3` funkcias kaj kongruas kun la oficiala vektoro.
fn kontroli_haketilon(r: &mut Rezultoj) {
    let malplena: Haketo = BLAKE3_MALPLENA
        .parse()
        .unwrap_or_else(|_| Haketo::el_bajtoj([0; 32]));
    r.kontroli(
        "blake3 malplena enigo kongruas kun la oficiala vektoro",
        haketi(b"") == malplena,
    );
    r.kontroli(
        "blake3 distingas malsamajn enigojn",
        haketi(b"a") != haketi(b"b"),
    );
}

/// Enmemora EAS: konservi kaj ricevi, plus difekto-rifuzo.
fn kontroli_konservilon(r: &mut Rezultoj) {
    let stokejo = MemoraKonservilo::nova();
    let datumoj: &[u8] = b"arm64-saluton";
    let Ok(haketo) = stokejo.konservi(datumoj) else {
        r.kontroli("konservi sukcesas", false);
        return;
    };

    r.kontroli(
        "konservi redonas la haketon de la datumoj",
        haketo == haketi(datumoj),
    );
    r.kontroli(
        "ricevi redonas la identajn datumojn",
        matches!(stokejo.ricevi(&haketo), Ok(ref enhavo) if enhavo.as_slice() == datumoj),
    );

    let mut difektitaj: Vec<u8> = datumoj.to_vec();
    if let Some(bito) = difektitaj.first_mut() {
        *bito ^= 1;
    }
    stokejo.enigi_krudajn(haketo, difektitaj);
    r.kontroli(
        "ricevi rifuzas difektitajn datumojn",
        matches!(stokejo.ricevi(&haketo), Err(Eraro::DatumoDifektita { .. })),
    );
}

/// Kanonika arbo: permuto ne influas la seriigon, duplikatoj rifuziĝas.
fn kontroli_arbon(r: &mut Rezultoj) {
    let haketo = Haketo::el_bajtoj([7; 32]);
    let dosiero = ArbaEnskribo::Dosiero {
        nomo: String::from("a"),
        haketo,
        grandeco: 5,
    };
    let subarbo = ArbaEnskribo::SubArbo {
        nomo: String::from("b"),
        haketo,
    };
    let unua = Arbo::el_enskriboj(vec![dosiero.clone(), subarbo.clone()]);
    let dua = Arbo::el_enskriboj(vec![subarbo, dosiero]);
    let kanonika = match (unua, dua) {
        (Ok(unua), Ok(dua)) => match (unua.seriigi(), dua.seriigi()) {
            (Ok(unua), Ok(dua)) => unua == dua,
            _ => false,
        },
        _ => false,
    };
    r.kontroli("permuto de enskriboj ne ŝanĝas la seriigon", kanonika);

    let duplikita = Arbo::el_enskriboj(vec![
        ArbaEnskribo::Dosiero {
            nomo: String::from("a"),
            haketo,
            grandeco: 0,
        },
        ArbaEnskribo::SubArbo {
            nomo: String::from("a"),
            haketo,
        },
    ]);
    r.kontroli(
        "duplikataj vojoj estas rifuzitaj",
        matches!(duplikita, Err(Eraro::DuplikataVojo { .. })),
    );
}

/// Voja validigo: danĝeraj komponantoj rifuziĝas.
fn kontroli_vojon(r: &mut Rezultoj) {
    r.kontroli("«..» rifuziĝas", validi_bajtojn(b"..").is_err());
    r.kontroli("«/» rifuziĝas", validi_bajtojn(b"a/b").is_err());
    r.kontroli("NUL-signo rifuziĝas", validi_bajtojn(b"a\0b").is_err());
    r.kontroli(
        "normala nomo akceptiĝas",
        validi_bajtojn(b"testo.txt").is_ok(),
    );
}

/// Limigita malseriigado: malplena arbo dekodiĝas, postrestaj bajtoj rifuziĝas.
fn kontroli_dekodon(r: &mut Rezultoj) {
    r.kontroli("malplena arbo dekodiĝas", Arbo::malseriigi(&[0]).is_ok());
    r.kontroli(
        "postrestaj bajtoj rifuziĝas",
        Arbo::malseriigi(&[0, 0]).is_err(),
    );
    r.kontroli(
        "trolonga prefikso rifuziĝas sen paniko",
        Arbo::malseriigi(&[0xff; 16]).is_err(),
    );
}

#[panic_handler]
fn paniko(informo: &core::panic::PanicInfo<'_>) -> ! {
    semihosting::println!("PANIKO: {informo}");
    semihosting::process::exit(101)
}

/// Simpla «bump»-alokilo: la tuta programo estas unu-fadena, la memorujo
/// estas fiksa kaj nenio estas liberigata.
mod stako {
    // PERMESO-JUSTIFIKO: bare-metal sen-OS medio ne havas malloc; `GlobalAlloc`
    // devige estas `unsafe`-trajto, la alokilo estas izolita en ĉi tiu modulo.
    #![allow(
        unsafe_code,
        reason = "bare-metal sen-OS medio postulas unsafe GlobalAlloc-realigon"
    )]
    // PERMESO-JUSTIFIKO: la kampo estas alirebla nur per kruda montrilo,
    // do la kompililo ne povas vidi legon de ĝi.
    #![allow(dead_code, reason = "la bufro estas alirata nur per kruda montrilo")]

    use core::alloc::{GlobalAlloc, Layout};
    use core::ptr;
    use core::sync::atomic::{AtomicUsize, Ordering};

    const LONGO: usize = 2 << 20;

    #[repr(align(16))]
    struct Hardo([u8; LONGO]);

    static mut HARDO: Hardo = Hardo([0; LONGO]);
    static PLENO: AtomicUsize = AtomicUsize::new(0);

    /// La alokilo mem.
    pub(crate) struct StakaAlokilo;

    // SAFETY: la alokilo nur antaŭeniras en unu fadeno kaj neniam liberigas,
    // do du alokoj neniam interkovriĝas.
    unsafe impl GlobalAlloc for StakaAlokilo {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let masko = layout.align().max(1).saturating_sub(1);
            let nun = PLENO.load(Ordering::Relaxed);
            let Some(start) = nun.checked_add(masko).map(|valoro| valoro & !masko) else {
                return ptr::null_mut();
            };
            let Some(fino) = start.checked_add(layout.size()) else {
                return ptr::null_mut();
            };
            if fino > LONGO {
                return ptr::null_mut();
            }
            PLENO.store(fino, Ordering::Relaxed);
            // SAFETY: la intervalo [start, fino) kuŝas ene de HARDO, kiu
            // ekzistas dum la tuta programo kaj estas alirebla nur ĉi tie.
            unsafe { ptr::addr_of_mut!(HARDO).cast::<u8>().add(start) }
        }

        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
    }
}

#[global_allocator]
static ALOKILO: stako::StakaAlokilo = stako::StakaAlokilo;

/// Enirpunkto kaj stakstarigo — la sola platforma gluo de la binaro.
mod eniro {
    // PERMESO-JUSTIFIKO: enirpunkto de bare-metal binaro devas uzi
    // `global_asm!` kaj `unsafe(export_name)`; la testa logiko restas ekster
    // ĉi tiu modulo kaj plu sub la malpermeso de `unsafe_code`.
    #![allow(
        unsafe_code,
        reason = "bare-metal enirpunkto postulas global_asm! kaj export_name"
    )]

    /// Vokas la ĉefan testrutinon sub la nomo, kiun atendas `_start`.
    #[unsafe(export_name = "rust_main")]
    pub(crate) fn ekiri() -> ! {
        super::rust_main()
    }

    core::arch::global_asm!(
        r#"
        .section .text._start, "ax"
        .global _start
    _start:
        ldr x0, =__stack_top
        mov sp, x0
        /* FP/SIMD estas malŝaltita post rekta `-kernel` starto; Rust uzas
           NEON, do ni devas malfermi CPACR_EL1.FPEN. */
        mrs x2, CPACR_EL1
        orr x2, x2, #(0b11 << 20)
        msr CPACR_EL1, x2
        isb
        ldr x0, =__bss_start
        ldr x1, =__bss_end
    1:
        cmp x0, x1
        b.hs 2f
        str xzr, [x0], #8
        b 1b
    2:
        bl rust_main
    3:
        b 3b
        "#
    );
}
