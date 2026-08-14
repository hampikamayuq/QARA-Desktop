'use strict';

/*
 * QARA Desktop — camada de UI.
 *
 * Contrato com o host nativo: docs/PROTOCOL.md (v1).
 * A UI nunca envia URL nem executável ao host: só IDs lógicos da config validada.
 * Nada de conteúdo sensível em logs.
 */

/** Versão do protocolo host ↔ UI. */
const PROTOCOL_VERSION = 1;

/** Tempo máximo de espera por uma resposta do host, em ms. */
const REQUEST_TIMEOUT_MS = 5000;

/** Prefixo das preferências no localStorage (usado apenas no modo preview). */
const PREF_STORAGE_PREFIX = 'qara.prefs.';

/** Textos institucionais padrão, usados quando `config.branding` está ausente. */
const DEFAULT_BRANDING = {
  kicker: 'CLÍNICA DERMATOLÓGICA',
  wordmark: 'QARA',
  headline: ['Conhecimento que cuida.', 'Presença que transforma.'],
  tagline: 'Precisão dermatológica com presença humana.',
  location: 'COPACABANA · RIO DE JANEIRO'
};

/** Config de fallback do modo preview (sem host). Espelha config/shortcuts.example.json. */
const FALLBACK_CONFIG = {
  profile: 'reception',
  locale: 'pt-BR',
  branding: DEFAULT_BRANDING,
  groups: [
    {
      id: 'attendance',
      label: 'Atendimento',
      items: [
        { id: 'doctoralia', label: 'Doctoralia', type: 'url', target: 'https://www.doctoralia.com.br/' },
        { id: 'kommo', label: 'Kommo', type: 'url', target: 'https://www.kommo.com/' },
        { id: 'whatsapp', label: 'WhatsApp', type: 'url', target: 'https://web.whatsapp.com/' }
      ]
    },
    {
      id: 'organization',
      label: 'Organização',
      items: [
        { id: 'gmail', label: 'Gmail', type: 'url', target: 'https://mail.google.com/' },
        { id: 'calendar', label: 'Agenda', type: 'url', target: 'https://calendar.google.com/' },
        { id: 'files', label: 'Arquivos', type: 'app', target: 'files' }
      ]
    },
    {
      id: 'qara',
      label: 'QARA',
      items: [
        { id: 'qara-site', label: 'Site QARA', type: 'url', target: 'https://clinicaqara.com.br/' },
        { id: 'qara-crm', label: 'CRM QARA', type: 'url', target: 'https://cliniqara-crm.onrender.com/' }
      ]
    }
  ],
  app_allowlist: { files: ['cosmic-files'] }
};

/**
 * Mensagens de erro em pt-BR para os códigos de docs/PROTOCOL.md.
 * Discretas e sem detalhe técnico — o desktop é visível a terceiros.
 */
const ERROR_MESSAGES = {
  invalid_message: 'Não foi possível concluir a ação.',
  unknown_action: 'Ação não reconhecida pelo sistema.',
  unknown_id: 'Atalho não encontrado na configuração.',
  not_allowlisted: 'Este atalho não está autorizado.',
  launch_failed: 'Não foi possível abrir o aplicativo.',
  invalid_preference: 'Não foi possível salvar a preferência.',
  timeout: 'O sistema não respondeu. Tente novamente.',
  bridge_unavailable: 'Ponte com o sistema indisponível.'
};

const DEFAULT_ERROR_MESSAGE = 'Não foi possível concluir a ação.';

/* ------------------------------------------------------------------ *
 * Ícones dos atalhos
 * ------------------------------------------------------------------ */

/**
 * Ícones SVG monocromáticos, linha fina, herdando `currentColor`.
 * Markup estático e confiável (nunca dados da config) — ver tarefa P3.
 */
const SHORTCUT_ICONS = {
  // Estetoscópio — perfil médico / agendamento.
  doctoralia:
    '<path d="M6 3v6a4 4 0 0 0 8 0V3"/><path d="M4 3h4"/><path d="M12 3h4"/>' +
    '<path d="M10 13v2a4 4 0 0 0 8 0v-1.5"/><circle cx="18" cy="11" r="2.5"/>',
  // Funil — pipeline comercial.
  kommo: '<path d="M3.5 5h17l-6.5 7.5V19l-4 2v-8.5z"/>',
  // Balão de conversa.
  whatsapp: '<path d="M20.5 11.6a8 8 0 0 1-11.7 7.1L4 20l1.4-4.2A8 8 0 1 1 20.5 11.6z"/>',
  // Envelope.
  gmail: '<rect x="3" y="5" width="18" height="14" rx="2"/><path d="m3.8 6.6 8.2 6.2 8.2-6.2"/>',
  // Calendário.
  calendar:
    '<rect x="3" y="5" width="18" height="16" rx="2"/><path d="M3 10h18"/>' +
    '<path d="M8 3v4"/><path d="M16 3v4"/>',
  // Pasta.
  files: '<path d="M3 7a2 2 0 0 1 2-2h3.6l2 2.5H19a2 2 0 0 1 2 2V17a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  // Globo — site institucional.
  'qara-site':
    '<circle cx="12" cy="12" r="9"/><path d="M3 12h18"/>' +
    '<path d="M12 3a13.5 13.5 0 0 1 0 18 13.5 13.5 0 0 1 0-18z"/>',
  // Ficha de contato — CRM/prontuário.
  'qara-crm':
    '<rect x="3" y="5" width="18" height="14" rx="2"/><circle cx="8.8" cy="11" r="2.2"/>' +
    '<path d="M5.6 16.3a3.4 3.4 0 0 1 6.4 0"/><path d="M14.6 10.4h3.8"/><path d="M14.6 13.6h3.8"/>'
};

/**
 * Monta o SVG inline de um atalho.
 * @param {string} id ID lógico do item.
 * @returns {string|null} markup do SVG ou null quando o ID é desconhecido.
 */
function shortcutIconMarkup(id) {
  const paths = Object.prototype.hasOwnProperty.call(SHORTCUT_ICONS, id) ? SHORTCUT_ICONS[id] : null;
  if (!paths) {
    return null;
  }
  return (
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" ' +
    'stroke-linecap="round" stroke-linejoin="round" focusable="false" aria-hidden="true">' +
    paths +
    '</svg>'
  );
}

/* ------------------------------------------------------------------ *
 * Estado da UI
 * ------------------------------------------------------------------ */

const $ = (selector) => document.querySelector(selector);

/** Config em uso (do host ou fallback do preview). */
let activeConfig = FALLBACK_CONFIG;

/** Índice id → item, para resolver ações no modo preview. */
let itemsById = new Map();

let toastTimer = null;
let clockTimeout = null;
let clockInterval = null;

/**
 * Mostra um toast discreto.
 * @param {string} message texto já em pt-BR.
 */
function showToast(message) {
  const toast = $('#toast');
  if (!toast) {
    return;
  }
  toast.textContent = message;
  toast.classList.add('visible');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toast.classList.remove('visible'), 2600);
}

/**
 * Traduz um código de erro do protocolo para mensagem de usuário.
 * @param {unknown} error
 * @returns {string}
 */
function errorMessage(error) {
  const code = error && typeof error.message === 'string' ? error.message : '';
  return Object.prototype.hasOwnProperty.call(ERROR_MESSAGES, code)
    ? ERROR_MESSAGES[code]
    : DEFAULT_ERROR_MESSAGE;
}

/* ------------------------------------------------------------------ *
 * Preferências (modo preview)
 * ------------------------------------------------------------------ */

/**
 * Lê uma preferência do localStorage. Usado só sem host.
 * @param {string} key
 * @returns {unknown}
 */
function readLocalPreference(key) {
  try {
    const raw = window.localStorage.getItem(PREF_STORAGE_PREFIX + key);
    return raw === null ? undefined : JSON.parse(raw);
  } catch (_error) {
    return undefined;
  }
}

/**
 * Grava uma preferência no localStorage. Usado só sem host.
 * @param {string} key
 * @param {unknown} value
 * @returns {boolean} sucesso
 */
function writeLocalPreference(key, value) {
  try {
    window.localStorage.setItem(PREF_STORAGE_PREFIX + key, JSON.stringify(value));
    return true;
  } catch (_error) {
    return false;
  }
}

/* ------------------------------------------------------------------ *
 * QaraBridge — ponte UI ↔ host (docs/PROTOCOL.md v1)
 * ------------------------------------------------------------------ */

const QaraBridge = (() => {
  /** Requisições aguardando resposta do host, por req_id. */
  const pending = new Map();
  let sequence = 0;

  /** @returns {boolean} true quando a UI roda dentro do host nativo. */
  function isHostAvailable() {
    return (
      window.__QARA_HOST__ === true &&
      Boolean(window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.qara)
    );
  }

  /** @returns {string} req_id único dentro desta sessão da WebView. */
  function nextRequestId() {
    sequence += 1;
    return `r-${sequence}`;
  }

  /**
   * Executa a ação localmente quando não há host (preview no navegador).
   * @param {string} action
   * @param {Record<string, unknown>} params
   * @returns {Promise<unknown>}
   */
  function previewInvoke(action, params) {
    switch (action) {
      case 'open_url': {
        const item = itemsById.get(params.id);
        if (!item || item.type !== 'url') {
          return Promise.reject(new Error('unknown_id'));
        }
        window.open(item.target, '_blank', 'noopener,noreferrer');
        return Promise.resolve(null);
      }
      case 'launch_app': {
        const item = itemsById.get(params.id);
        if (!item || item.type !== 'app') {
          return Promise.reject(new Error('unknown_id'));
        }
        showToast(`“${item.label}” abre pelo host nativo do QARA Desktop.`);
        return Promise.resolve(null);
      }
      case 'set_ui_preference': {
        if (typeof params.key !== 'string' || !writeLocalPreference(params.key, params.value)) {
          return Promise.reject(new Error('invalid_preference'));
        }
        return Promise.resolve(null);
      }
      case 'get_system_info':
        return Promise.resolve({
          hostname: 'preview',
          profile: typeof activeConfig.profile === 'string' ? activeConfig.profile : 'preview',
          version: 'preview'
        });
      default:
        return Promise.reject(new Error('unknown_action'));
    }
  }

  /**
   * Envia uma ação ao host e devolve uma Promise da resposta.
   * @param {string} action ação fechada do protocolo.
   * @param {Record<string, unknown>} [params] parâmetros (ex.: `{ id }`).
   * @returns {Promise<unknown>} resolve com `data`; rejeita com Error(código).
   */
  function invoke(action, params = {}) {
    if (!isHostAvailable()) {
      return previewInvoke(action, params);
    }

    return new Promise((resolve, reject) => {
      const reqId = nextRequestId();
      const timer = setTimeout(() => {
        pending.delete(reqId);
        reject(new Error('timeout'));
      }, REQUEST_TIMEOUT_MS);

      pending.set(reqId, { resolve, reject, timer });

      const message = Object.assign(
        { v: PROTOCOL_VERSION, req_id: reqId, action },
        params
      );

      try {
        window.webkit.messageHandlers.qara.postMessage(JSON.stringify(message));
      } catch (_error) {
        clearTimeout(timer);
        pending.delete(reqId);
        reject(new Error('bridge_unavailable'));
      }
    });
  }

  /**
   * Chamado pelo host para resolver/rejeitar uma requisição pendente.
   * @param {{ req_id?: string, ok?: boolean, data?: unknown, error?: string }} response
   */
  function onHostResponse(response) {
    if (!response || typeof response !== 'object') {
      return;
    }
    const entry = pending.get(response.req_id);
    if (!entry) {
      return;
    }
    pending.delete(response.req_id);
    clearTimeout(entry.timer);

    if (response.ok === true) {
      entry.resolve(response.data === undefined ? null : response.data);
      return;
    }
    const code = typeof response.error === 'string' ? response.error : 'invalid_message';
    entry.reject(new Error(code));
  }

  return {
    invoke,
    onHostResponse,
    isHostAvailable,
    /** @returns {number} requisições em voo (diagnóstico/teste). */
    pendingCount: () => pending.size
  };
})();

window.QaraBridge = QaraBridge;

/* ------------------------------------------------------------------ *
 * Relógio
 * ------------------------------------------------------------------ */

function updateClock() {
  const now = new Date();
  const clock = $('#clock');
  const date = $('#date');
  if (clock) {
    clock.textContent = new Intl.DateTimeFormat('pt-BR', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: false
    }).format(now);
  }
  if (date) {
    date.textContent = new Intl.DateTimeFormat('pt-BR', {
      weekday: 'long',
      day: '2-digit',
      month: 'long'
    }).format(now);
  }
}

/**
 * Alinha o relógio à virada do minuto: um setTimeout até o próximo minuto
 * e, a partir daí, um intervalo de 60 s (SPEC §8 — menos wakeups em idle).
 */
function startClock() {
  clearTimeout(clockTimeout);
  clearInterval(clockInterval);
  updateClock();

  const msToNextMinute = 60_000 - (Date.now() % 60_000);
  clockTimeout = setTimeout(() => {
    updateClock();
    clockInterval = setInterval(updateClock, 60_000);
  }, msToNextMinute);
}

/* ------------------------------------------------------------------ *
 * Branding
 * ------------------------------------------------------------------ */

/**
 * Popula os textos institucionais a partir de `config.branding`.
 * Sempre via textContent — nunca innerHTML para dados da config.
 * @param {Record<string, unknown>} branding
 */
function renderBranding(branding) {
  const source = branding && typeof branding === 'object' ? branding : {};
  const text = (key) =>
    typeof source[key] === 'string' && source[key].trim() !== '' ? source[key] : DEFAULT_BRANDING[key];

  const kicker = $('#brand-kicker');
  if (kicker) {
    kicker.textContent = text('kicker');
  }

  const wordmark = $('#brand-wordmark');
  if (wordmark) {
    wordmark.textContent = text('wordmark');
    wordmark.setAttribute('aria-label', text('wordmark'));
  }

  const tagline = $('#brand-tagline');
  if (tagline) {
    tagline.textContent = text('tagline');
  }

  const location = $('#brand-location');
  if (location) {
    location.textContent = text('location');
  }

  const headline = $('#brand-headline');
  if (headline) {
    const lines =
      Array.isArray(source.headline) && source.headline.length > 0
        ? source.headline.filter((line) => typeof line === 'string')
        : DEFAULT_BRANDING.headline;
    const safeLines = lines.length > 0 ? lines : DEFAULT_BRANDING.headline;

    headline.textContent = '';
    safeLines.forEach((line, index) => {
      const span = document.createElement('span');
      span.className = 'headline-line';
      if (index === safeLines.length - 1 && safeLines.length > 1) {
        span.classList.add('headline-line--accent');
      }
      span.textContent = line;
      headline.appendChild(span);
    });
  }
}

/* ------------------------------------------------------------------ *
 * Atalhos
 * ------------------------------------------------------------------ */

/**
 * Dispara a ação do atalho pela bridge (ou pelo fallback de preview).
 * @param {{id: string, label: string, type: string}} item
 */
function activate(item) {
  const action = item.type === 'url' ? 'open_url' : 'launch_app';
  QaraBridge.invoke(action, { id: item.id }).catch((error) => {
    showToast(errorMessage(error));
  });
}

/**
 * Cria o botão de um atalho.
 * @param {{id: string, label: string, type: string}} item
 * @returns {HTMLButtonElement}
 */
function createShortcut(item) {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'shortcut';
  button.dataset.id = item.id;

  const mark = document.createElement('span');
  mark.setAttribute('aria-hidden', 'true');
  const icon = shortcutIconMarkup(item.id);
  if (icon) {
    mark.className = 'shortcut-icon';
    // Markup estático definido neste arquivo; nenhum dado de config entra aqui.
    mark.innerHTML = icon;
  } else {
    // Fallback discreto para IDs desconhecidos: a barrinha taupe original.
    mark.className = 'shortcut-mark';
  }

  const label = document.createElement('span');
  label.className = 'shortcut-label';
  label.textContent = item.label;

  const type = document.createElement('span');
  type.className = 'shortcut-type';
  type.textContent = item.type === 'url' ? 'web' : 'aplicativo';

  button.append(mark, label, type);
  button.addEventListener('click', () => activate(item));
  return button;
}

/**
 * Renderiza os grupos de atalhos e reconstrói o índice id → item.
 * @param {Record<string, unknown>} config
 */
function renderShortcuts(config) {
  const root = $('#shortcut-groups');
  if (!root) {
    return;
  }
  root.textContent = '';
  itemsById = new Map();

  const groups = Array.isArray(config.groups) ? config.groups : [];
  groups.forEach((group) => {
    if (!group || !Array.isArray(group.items)) {
      return;
    }

    const section = document.createElement('section');
    section.className = 'shortcut-group';

    const label = document.createElement('div');
    label.className = 'group-label';
    label.textContent = typeof group.label === 'string' ? group.label : '';

    const items = document.createElement('div');
    items.className = 'group-items';

    group.items.forEach((item) => {
      if (!item || typeof item.id !== 'string' || typeof item.label !== 'string') {
        return;
      }
      itemsById.set(item.id, item);
      items.appendChild(createShortcut(item));
    });

    section.append(label, items);
    root.appendChild(section);
  });
}

/* ------------------------------------------------------------------ *
 * Preferência: modo compacto
 * ------------------------------------------------------------------ */

/**
 * Aplica o estado compacto ao body e ao botão.
 * @param {boolean} compact
 */
function applyCompact(compact) {
  document.body.classList.toggle('compact', compact);
  const toggle = $('#compact-toggle');
  if (toggle) {
    toggle.setAttribute('aria-pressed', String(compact));
    toggle.textContent = compact ? 'Expandir' : 'Compactar';
  }
}

/**
 * Liga o toggle e persiste a escolha (host: state.json; preview: localStorage).
 */
function setupCompactToggle() {
  const toggle = $('#compact-toggle');
  if (!toggle) {
    return;
  }
  toggle.addEventListener('click', () => {
    const compact = !document.body.classList.contains('compact');
    applyCompact(compact);
    QaraBridge.invoke('set_ui_preference', { key: 'compact', value: compact }).catch((error) => {
      showToast(errorMessage(error));
    });
  });
}

/**
 * Resolve as preferências iniciais: injetadas pelo host, ou localStorage no preview.
 * @returns {{compact: boolean}}
 */
function resolvePreferences() {
  const injected = window.__QARA_PREFS__;
  if (injected && typeof injected === 'object' && typeof injected.compact === 'boolean') {
    return { compact: injected.compact };
  }
  if (!QaraBridge.isHostAvailable()) {
    return { compact: readLocalPreference('compact') === true };
  }
  return { compact: false };
}

/* ------------------------------------------------------------------ *
 * Inicialização
 * ------------------------------------------------------------------ */

function init() {
  const injectedConfig = window.__QARA_CONFIG__;
  activeConfig =
    injectedConfig && typeof injectedConfig === 'object' && Array.isArray(injectedConfig.groups)
      ? injectedConfig
      : FALLBACK_CONFIG;

  document.body.classList.toggle('host-mode', QaraBridge.isHostAvailable());

  renderBranding(activeConfig.branding);
  renderShortcuts(activeConfig);
  setupCompactToggle();
  applyCompact(resolvePreferences().compact);
  startClock();
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init, { once: true });
} else {
  init();
}
