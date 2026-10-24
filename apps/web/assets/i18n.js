/**
 * The language runtime of the dashboard.
 *
 * WHAT: the single place that knows which language the panel speaks and
 * that writes the texts of the language file into the document.
 * WHY: the markup of the dashboard carries keys, not words. A word that is
 * written twice — once in the markup and once in a module — drifts, and a
 * word that is written in the markup cannot be translated at all. The
 * runtime reads the language file, replaces the placeholders and writes the
 * result into the elements that ask for it.
 * HOW: `t(key, params)` returns a text; `applyTranslations(root)` walks
 * the document and fills every element that carries `data-i18n`,
 * `data-i18n-content` or `data-i18n-attr`. The language is chosen once, at
 * the start: the query parameter `?lang=`, then the remembered choice, then
 * the language of the browser, then the language of the document, then
 * English.
 *
 * EXAMPLE:
 *
 * ```js
 * import { t, applyTranslations } from "./i18n.js";
 *
 * applyTranslations(document);          // fills every key of the markup
 * t("card.switch.on");                  // "On" — or "An" in German
 * t("session.count", { count: 2 });     // "2 sessions streaming."
 * ```
 */

import { MESSAGES as ENGLISH } from "./lang/en.js";
import { MESSAGES as GERMAN } from "./lang/de.js";

/** The languages the dashboard ships with. */
const LANGUAGES = { en: ENGLISH, de: GERMAN };

/** The language the dashboard falls back to. */
export const FALLBACK_LANGUAGE = "en";

/** The key under which the chosen language is remembered. */
const STORAGE_KEY = "phairplayrust-language";

/** The language that is spoken right now. */
let current = FALLBACK_LANGUAGE;

/**
 * Reads the language of the browser, if there is one.
 *
 * @returns {string} a language tag like "de" or "en-US", or an empty string
 */
function browserLanguage() {
  const navigatorObject = globalThis.navigator;
  if (!navigatorObject || typeof navigatorObject.language !== "string") return "";
  return navigatorObject.language;
}

/**
 * Reads the language the document declares.
 *
 * @returns {string}
 */
function documentLanguage() {
  if (typeof document === "undefined") return "";
  const root = document.documentElement;
  return root ? root.getAttribute("lang") || "" : "";
}

/**
 * Reads the language the query string asks for.
 *
 * @returns {string}
 */
function queryLanguage() {
  if (typeof location === "undefined" || !location.search) return "";
  try {
    return new URLSearchParams(location.search).get("lang") || "";
  } catch (error) {
    return "";
  }
}

/**
 * Reads the language that was chosen the last time.
 *
 * @returns {string}
 */
function rememberedLanguage() {
  try {
    return localStorage.getItem(STORAGE_KEY) || "";
  } catch (error) {
    /* A blocked storage must not break the panel. */
    return "";
  }
}

/**
 * Reduces a language tag to a language the dashboard ships with.
 *
 * WHY the reduction: the browser says "de-AT" and "de-CH", the panel ships
 * "de". The first two letters decide.
 *
 * @param {string} tag
 * @returns {string} the key of a language, or an empty string
 */
export function resolveLanguage(tag) {
  const normalized = String(tag || "").trim().toLowerCase();
  if (!normalized) return "";
  if (LANGUAGES[normalized]) return normalized;
  const short = normalized.split(/[-_]/)[0];
  return LANGUAGES[short] ? short : "";
}

/**
 * Chooses the language of the panel.
 *
 * WHY this order: an explicit choice of the user wins over everything, the
 * remembered choice comes next, and the language of the browser is only
 * consulted when the user has never chosen one.
 *
 * @returns {string} the key of the language that is spoken now
 */
export function detectLanguage() {
  const candidates = [
    queryLanguage(),
    rememberedLanguage(),
    browserLanguage(),
    documentLanguage(),
    FALLBACK_LANGUAGE,
  ];
  for (const candidate of candidates) {
    const language = resolveLanguage(candidate);
    if (language) return language;
  }
  return FALLBACK_LANGUAGE;
}

/**
 * Switches the panel to another language and remembers the choice.
 *
 * @param {string} code the key of a language, e.g. "de"
 * @returns {string} the language that is spoken now
 */
export function setLanguage(code) {
  const language = resolveLanguage(code) || FALLBACK_LANGUAGE;
  current = language;
  try {
    localStorage.setItem(STORAGE_KEY, language);
  } catch (error) {
    /* A blocked storage must not break the panel. */
  }
  if (typeof document !== "undefined" && document.documentElement) {
    document.documentElement.setAttribute("lang", language);
  }
  return current;
}

/**
 * The language that is spoken right now.
 *
 * @returns {string}
 */
export function language() {
  return current;
}

/**
 * The languages the dashboard ships with.
 *
 * @returns {string[]}
 */
export function availableLanguages() {
  return Object.keys(LANGUAGES);
}

/**
 * Returns the text of a key in the language that is spoken right now.
 *
 * WHY the fallback to English: a language file that is not complete must
 * not blank the panel. A key that is missing shows its English text.
 *
 * @param {string} key the key of the text
 * @param {Record<string, unknown>} [params] the values of the placeholders
 * @returns {string}
 */
export function t(key, params = {}) {
  const table = LANGUAGES[current] || ENGLISH;
  const template = table[key] ?? ENGLISH[key] ?? key;
  return template.replace(/\{(\w+)\}/g, (match, name) => {
    if (name in params && params[name] !== undefined && params[name] !== null) {
      return String(params[name]);
    }
    return match;
  });
}

/**
 * Writes the texts of the language file into the elements that ask for them.
 *
 * The three attributes of the markup:
 *   `data-i18n="key"`            the text of the element
 *   `data-i18n-content="key"`    the content attribute, e.g. of a meta tag
 *   `data-i18n-attr="title:key"` an attribute, several separated by a comma
 *
 * @param {ParentNode} [root] the part of the document that is filled
 * @returns {number} the number of elements that were filled
 */
export function applyTranslations(root = typeof document !== "undefined" ? document : null) {
  if (!root) return 0;
  let filled = 0;
  for (const element of root.querySelectorAll("[data-i18n]")) {
    element.textContent = t(element.getAttribute("data-i18n"));
    filled += 1;
  }
  for (const element of root.querySelectorAll("[data-i18n-content]")) {
    element.setAttribute("content", t(element.getAttribute("data-i18n-content")));
    filled += 1;
  }
  for (const element of root.querySelectorAll("[data-i18n-attr]")) {
    for (const pair of element.getAttribute("data-i18n-attr").split(",")) {
      const [attribute, key] = pair.split(":").map((part) => part.trim());
      if (!attribute || !key) continue;
      element.setAttribute(attribute, t(key));
    }
    filled += 1;
  }
  return filled;
}

/**
 * Chooses the language of the panel and fills the document with it.
 *
 * WHY the query parameter is not remembered: `?lang=de` is the language of
 * *this* address — a link that a colleague sent, a bookmark of a German
 * page. A choice that the user makes in the panel is remembered; a
 * parameter in the address is not.
 *
 * @returns {string} the language that is spoken now
 */
export function startLanguage() {
  const requested = resolveLanguage(queryLanguage());
  if (requested) {
    current = requested;
    if (typeof document !== "undefined" && document.documentElement) {
      document.documentElement.setAttribute("lang", current);
    }
  } else {
    setLanguage(detectLanguage());
  }
  applyTranslations(document);
  return current;
}

current = detectLanguage();
