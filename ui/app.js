const fallbackConfig = {
  groups: [
    { label: 'Atendimento', items: [
      { id: 'doctoralia', label: 'Doctoralia', type: 'url', target: 'https://www.doctoralia.com.br/' },
      { id: 'kommo', label: 'Kommo', type: 'url', target: 'https://www.kommo.com/' },
      { id: 'whatsapp', label: 'WhatsApp', type: 'url', target: 'https://web.whatsapp.com/' }
    ]},
    { label: 'Organização', items: [
      { id: 'gmail', label: 'Gmail', type: 'url', target: 'https://mail.google.com/' },
      { id: 'calendar', label: 'Agenda', type: 'url', target: 'https://calendar.google.com/' },
      { id: 'files', label: 'Arquivos', type: 'app', target: 'files' }
    ]},
    { label: 'QARA', items: [
      { id: 'qara-site', label: 'Site QARA', type: 'url', target: 'https://clinicaqara.com.br/' },
      { id: 'qara-crm', label: 'CRM QARA', type: 'url', target: 'https://cliniqara-crm.onrender.com/' }
    ]}
  ]
};

const $ = (selector) => document.querySelector(selector);
let toastTimer;

function showToast(message) {
  const toast = $('#toast');
  toast.textContent = message;
  toast.classList.add('visible');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toast.classList.remove('visible'), 2200);
}

function updateClock() {
  const now = new Date();
  $('#clock').textContent = new Intl.DateTimeFormat('pt-BR', {
    hour: '2-digit', minute: '2-digit', hour12: false
  }).format(now);
  $('#date').textContent = new Intl.DateTimeFormat('pt-BR', {
    weekday: 'long', day: '2-digit', month: 'long'
  }).format(now);
}

function nativeBridgeAvailable() {
  return Boolean(window.qaraNative && typeof window.qaraNative.invoke === 'function');
}

function activate(item) {
  if (nativeBridgeAvailable()) {
    window.qaraNative.invoke({ action: item.type === 'url' ? 'open_url' : 'launch_app', id: item.id });
    return;
  }

  if (item.type === 'url') {
    window.open(item.target, '_blank', 'noopener,noreferrer');
    return;
  }

  showToast(`“${item.label}” será aberto pelo host nativo.`);
}

function render(config) {
  const root = $('#shortcut-groups');
  root.innerHTML = '';

  config.groups.forEach((group) => {
    const section = document.createElement('section');
    section.className = 'shortcut-group';

    const label = document.createElement('div');
    label.className = 'group-label';
    label.textContent = group.label;

    const items = document.createElement('div');
    items.className = 'group-items';

    group.items.forEach((item) => {
      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'shortcut';
      button.dataset.id = item.id;
      button.innerHTML = `
        <span class="shortcut-mark" aria-hidden="true"></span>
        <span class="shortcut-label"></span>
        <span class="shortcut-type"></span>
      `;
      button.querySelector('.shortcut-label').textContent = item.label;
      button.querySelector('.shortcut-type').textContent = item.type === 'url' ? 'web' : 'aplicativo';
      button.addEventListener('click', () => activate(item));
      items.appendChild(button);
    });

    section.append(label, items);
    root.appendChild(section);
  });
}

$('#compact-toggle').addEventListener('click', (event) => {
  const compact = document.body.classList.toggle('compact');
  event.currentTarget.setAttribute('aria-pressed', String(compact));
  event.currentTarget.textContent = compact ? 'Expandir' : 'Compactar';
});

render(fallbackConfig);
updateClock();
setInterval(updateClock, 30_000);
