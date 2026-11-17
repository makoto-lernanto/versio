"use strict";

// Konduto de la Versio-retejo: lingvo, temo, akcentoj, fenestroj, kopiado
// kaj la FluoU-movoj. La fonta lingvo estas Esperanto; la angla venas nur
// el `tradukoj/angla.js` kiel aparta mapada tavolo.

(() => {
  const radiko = document.documentElement;
  const VERSIO = radiko.dataset.versio || "0.1.0";
  const ANGLA = window.TRADUKOJ_ANGLA || {};
  const reduktitaMovo = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  const AKCENTOJ = {
    oro: "#FFB454",
    koralo: "#FF5C8A",
    cxielo: "#4CC9FF",
    verdo: "#57E389",
    viola: "#A78BFA",
    bluo: "#5B7CFA",
  };
  const AKCENTAJ_NOMOJ = {
    oro: "oro",
    koralo: "koralo",
    cxielo: "ĉielo",
    verdo: "verdo",
    viola: "viola",
    bluo: "bluo",
  };

  const $ = (selektilo, patro = document) => patro.querySelector(selektilo);
  const $$ = (selektilo, patro = document) => Array.from(patro.querySelectorAll(selektilo));

  let lingvo = "eo";
  const originalaTitolo = document.title;
  const originaloj = new Map();

  function traduki(ŝlosilo, originalo) {
    if (lingvo === "en" && typeof ANGLA[ŝlosilo] === "string") {
      return ANGLA[ŝlosilo];
    }
    return originalo;
  }

  function t(ŝlosilo, defaŭlto) {
    return traduki(ŝlosilo, defaŭlto ?? ŝlosilo);
  }

  function legi(ŝlosilo, defaŭlto) {
    try {
      return localStorage.getItem(ŝlosilo) ?? defaŭlto;
    } catch {
      return defaŭlto;
    }
  }

  function skribi(ŝlosilo, valoro) {
    try {
      localStorage.setItem(ŝlosilo, valoro);
    } catch {
      // La retejo restas funkcianta ankaŭ sen konservado.
    }
  }

  function konserviOriginalojn() {
    $$(
      "[data-t], [data-t-html], [data-t-aria], [data-t-meta], [data-t-title]"
    ).forEach((elemento) => {
      originaloj.set(elemento, {
        teksto: elemento.textContent,
        html: elemento.innerHTML,
        aria: elemento.getAttribute("aria-label"),
        meta: elemento.getAttribute("content"),
        titolo: elemento.getAttribute("title"),
      });
    });
  }

  function aplikiLingvon(nova, konservu = true) {
    lingvo = nova === "en" ? "en" : "eo";
    radiko.lang = lingvo;

    originaloj.forEach((originalo, elemento) => {
      const datumoj = elemento.dataset;
      if (datumoj.t !== undefined) {
        elemento.textContent = traduki(datumoj.t, originalo.teksto);
      }
      if (datumoj.tHtml !== undefined) {
        elemento.innerHTML = traduki(datumoj.tHtml, originalo.html);
      }
      if (datumoj.tAria !== undefined) {
        elemento.setAttribute("aria-label", traduki(datumoj.tAria, originalo.aria));
      }
      if (datumoj.tMeta !== undefined) {
        elemento.setAttribute("content", traduki(datumoj.tMeta, originalo.meta));
      }
      if (datumoj.tTitle !== undefined) {
        elemento.setAttribute("title", traduki(datumoj.tTitle, originalo.titolo));
      }
    });

    document.title = traduki("meta.titolo", originalaTitolo);
    $$("[data-lingvo]").forEach((butono) => {
      butono.setAttribute("aria-pressed", butono.dataset.lingvo === lingvo ? "true" : "false");
    });

    agordiTemon(radiko.dataset.temo || "malluma", false);
    agordiAkcenton(radiko.dataset.akcento || "verdo", false);
    gxisdatigiFenestrojn();

    if (konservu) {
      skribi("versio-lingvo", lingvo);
    }
  }

  function agordiTemon(temo, konservu = true) {
    const valoro = temo === "hela" ? "hela" : "malluma";
    radiko.dataset.temo = valoro;

    const ŝaltilo = $("#temoSxaltilo");
    if (ŝaltilo) {
      ŝaltilo.checked = valoro === "malluma";
      ŝaltilo.setAttribute(
        "aria-label",
        t(
          valoro === "malluma" ? "temo.aria.malluma" : "temo.aria.hela",
          valoro === "malluma" ? "Malluma temo" : "Hela temo"
        )
      );
    }

    const etikedo = $("#temoEtikedo");
    if (etikedo) {
      etikedo.textContent = t(
        valoro === "malluma" ? "temo.malluma" : "temo.hela",
        valoro === "malluma" ? "Malluma" : "Hela"
      );
    }

    const koloro = valoro === "malluma" ? "#0A0D1C" : "#F6F3EC";
    $('meta[name="theme-color"]')?.setAttribute("content", koloro);

    if (konservu) {
      skribi("versio-temo", valoro);
    }
  }

  function agordiAkcenton(nomo, konservu = true) {
    const valoro = Object.prototype.hasOwnProperty.call(AKCENTOJ, nomo) ? nomo : "verdo";
    radiko.dataset.akcento = valoro;
    radiko.style.setProperty("--akcento", AKCENTOJ[valoro]);

    $$("[data-akcento]").forEach((butono) => {
      const akcento = butono.dataset.akcento;
      const aktiva = akcento === valoro;
      const nomoEnLingvo = AKCENTAJ_NOMOJ[akcento] || akcento;
      const etikedo = t(`akcento.${akcento}`, `Akcento: ${nomoEnLingvo}`);

      butono.setAttribute("aria-pressed", aktiva ? "true" : "false");
      butono.setAttribute("aria-label", etikedo);
      butono.setAttribute("title", etikedo);
    });

    if (konservu) {
      skribi("versio-akcento", valoro);
    }
  }

  function gxisdatigiVersion() {
    $$("[data-versio-montri]").forEach((elemento) => {
      elemento.textContent = VERSIO;
    });
  }

  function gxisdatigiFenestrojn() {
    $$("[data-fenestro]").forEach((fenestro) => {
      const maksimuma = $('[data-ago="pligrandigi"]', fenestro);
      if (!maksimuma) {
        return;
      }
      const pligrandigita = fenestro.classList.contains("pligrandigita");
      const etikedo = t(
        pligrandigita ? "fenestro.restarigi" : "fenestro.pligrandigi",
        pligrandigita ? "Restarigi" : "Pligrandigi"
      );
      maksimuma.setAttribute("aria-label", etikedo);
      maksimuma.setAttribute("title", etikedo);
      maksimuma.setAttribute("aria-pressed", pligrandigita ? "true" : "false");
    });
  }

  function sxaltiFenestrojn() {
    $$("[data-fenestro]").forEach((fenestro) => {
      const identigilo = fenestro.dataset.fenestro;
      const sceno = fenestro.closest("[data-scene]");
      const restarigo = sceno
        ? $(`[data-restarigi="${identigilo}"]`, sceno)
        : null;

      const kasxi = (klaso) => {
        fenestro.classList.remove("pligrandigita");
        fenestro.classList.add(klaso);
        fenestro.inert = true;
        fenestro.setAttribute("aria-hidden", "true");
        if (restarigo) {
          restarigo.hidden = false;
          $("[data-restarigi-butono]", restarigo)?.focus();
        }
      };

      $$("[data-ago]", fenestro).forEach((butono) => {
        butono.addEventListener("click", () => {
          const ago = butono.dataset.ago;
          if (ago === "pligrandigi") {
            fenestro.classList.toggle("pligrandigita");
            gxisdatigiFenestrojn();
          } else if (ago === "malgrandigi") {
            kasxi("malgrandigita");
          } else if (ago === "fermi") {
            kasxi("fermita");
          }
        });
      });

      const titolstrio = $("[data-treni]", fenestro);
      if (titolstrio) {
        titolstrio.addEventListener("dblclick", (evento) => {
          if (evento.target.closest("button")) {
            return;
          }
          fenestro.classList.toggle("pligrandigita");
          gxisdatigiFenestrojn();
        });
      }

      if (restarigo) {
        $("[data-restarigi-butono]", restarigo)?.addEventListener("click", () => {
          fenestro.classList.remove("malgrandigita", "fermita", "pligrandigita");
          fenestro.inert = false;
          fenestro.removeAttribute("aria-hidden");
          restarigo.hidden = true;
          gxisdatigiFenestrojn();
          $('[data-ago="malgrandigi"]', fenestro)?.focus();
        });
      }
    });
  }

  async function kopiiTekston(teksto) {
    try {
      await navigator.clipboard.writeText(teksto);
      return true;
    } catch {
      const helpo = document.createElement("textarea");
      helpo.value = teksto;
      helpo.setAttribute("readonly", "");
      helpo.style.position = "fixed";
      helpo.style.opacity = "0";
      document.body.appendChild(helpo);
      helpo.select();
      let sukceso = false;
      try {
        sukceso = document.execCommand("copy");
      } catch {
        sukceso = false;
      }
      helpo.remove();
      return sukceso;
    }
  }

  function fulmomesagxo(titolo, teksto, semantiko = "sukceso") {
    const ujo = $("#fulmomesagxoj");
    if (!ujo) {
      return;
    }
    const mesaĝo = document.createElement("div");
    mesaĝo.className = `fulmomesagxo ${semantiko}`;

    const titola = document.createElement("div");
    titola.className = "fulma-titolo";
    titola.textContent = titolo;

    const teksta = document.createElement("div");
    teksta.className = "fulma-teksto";
    teksta.textContent = teksto;

    mesaĝo.append(titola, teksta);
    ujo.appendChild(mesaĝo);
    while (ujo.children.length > 4) {
      ujo.firstElementChild.remove();
    }
    window.setTimeout(() => {
      mesaĝo.classList.add("foriri");
      window.setTimeout(() => mesaĝo.remove(), 240);
    }, 3400);
  }

  function sxaltiKopiojn() {
    $$("[data-kopii]").forEach((butono) => {
      butono.addEventListener("click", async () => {
        const celo = document.getElementById(butono.dataset.kopii);
        if (!celo) {
          return;
        }
        const sukceso = await kopiiTekston(celo.textContent.trim());
        fulmomesagxo(
          sukceso ? t("fulmo.kopiita", "Kopiita") : t("fulmo.eraro", "Kopio malsukcesis"),
          sukceso
            ? t("fulmo.kopiita.teksto", "La teksto estas en la tondujo.")
            : t("fulmo.eraro.teksto", "Vi povas elekti la tekston permane."),
          sukceso ? "sukceso" : "dangxero"
        );
      });
    });
  }

  function sxaltiRevelon() {
    if (reduktitaMovo) {
      return;
    }
    $$(".revelo").forEach((elemento) => {
      elemento.addEventListener("pointermove", (evento) => {
        const kadro = elemento.getBoundingClientRect();
        if (!kadro.width || !kadro.height) {
          return;
        }
        const x = ((evento.clientX - kadro.left) / kadro.width) * 100;
        const y = ((evento.clientY - kadro.top) / kadro.height) * 100;
        elemento.style.setProperty("--mx", `${x.toFixed(1)}%`);
        elemento.style.setProperty("--my", `${y.toFixed(1)}%`);
      });
    });
  }

  function sxaltiAperon() {
    const eroj = $$("[data-aperu]");
    if (reduktitaMovo || !("IntersectionObserver" in window)) {
      eroj.forEach((elemento) => elemento.classList.add("videbla"));
      return;
    }

    eroj.forEach((elemento) => elemento.classList.add("aperu"));
    const observanto = new IntersectionObserver(
      (eniroj, observilo) => {
        eniroj.forEach((eniro) => {
          if (eniro.isIntersecting) {
            eniro.target.classList.add("videbla");
            observilo.unobserve(eniro.target);
          }
        });
      },
      { rootMargin: "0px 0px -6% 0px", threshold: 0.08 }
    );
    eroj.forEach((elemento) => observanto.observe(elemento));
  }

  function starti() {
    konserviOriginalojn();
    gxisdatigiVersion();

    const komencaLingvo = legi("versio-lingvo", "eo") === "en" ? "en" : "eo";
    const komencaTemo = legi("versio-temo", "malluma") === "hela" ? "hela" : "malluma";
    const komencaAkcento = legi("versio-akcento", "verdo");

    $("#temoSxaltilo")?.addEventListener("change", (evento) => {
      agordiTemon(evento.target.checked ? "malluma" : "hela");
    });
    $$("[data-akcento]").forEach((butono) => {
      butono.addEventListener("click", () => agordiAkcenton(butono.dataset.akcento));
    });
    $$("[data-lingvo]").forEach((butono) => {
      butono.addEventListener("click", () => aplikiLingvon(butono.dataset.lingvo));
    });

    sxaltiFenestrojn();
    sxaltiKopiojn();
    sxaltiRevelon();
    sxaltiAperon();

    agordiTemon(komencaTemo, false);
    agordiAkcenton(komencaAkcento, false);
    aplikiLingvon(komencaLingvo, false);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", starti);
  } else {
    starti();
  }
})();
