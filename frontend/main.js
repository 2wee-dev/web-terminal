import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import 'xterm/css/xterm.css';

const VERSION = '0.4';

const INTERCEPT_KEYS = new Set([
  'ctrl+s', 'ctrl+n', 'ctrl+d', 'ctrl+r', 'ctrl+l', 'ctrl+a', 'ctrl+o',
  'ctrl+shift+j', 'ctrl+shift+u', 'ctrl+shift+i',
  'ctrl+enter', 'shift+enter', 'alt+enter',
  'alt+arrowdown', 'alt+arrowup', 'alt+arrowleft', 'alt+arrowright',
  'f1', 'f2', 'f3', 'f4', 'f5', 'f6', 'f7', 'f8', 'f9', 'f10', 'f11', 'f12',
  'ctrl+=', 'ctrl+-', 'ctrl+0',
]);

function keyId(e) {
  const parts = [];
  if (e.ctrlKey || e.metaKey) parts.push('ctrl');
  if (e.altKey) parts.push('alt');
  if (e.shiftKey) parts.push('shift');
  parts.push(e.key.toLowerCase());
  return parts.join('+');
}

function installKeyInterception(getWs, getTerm, getFitAddon) {
  document.addEventListener('keydown', function (e) {
    const id = keyId(e);
    if (INTERCEPT_KEYS.has(id)) {
      e.preventDefault();
    }
    // Ctrl+Enter / Shift+Enter → send Alt+Enter sequence (\x1b\r)
    if (e.key === 'Enter' && (e.ctrlKey || e.shiftKey || e.metaKey)) {
      e.preventDefault();
      e.stopPropagation();
      const ws = getWs();
      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(new TextEncoder().encode('\x1b\r'));
      }
    }
    // Cmd/Ctrl + Plus/Minus/0 → zoom terminal font, not browser
    // Note: Cmd++ is Shift+= on most keyboards; Chrome blocks Cmd+Shift+= so we
    // also support Cmd+= (no shift) which does reach the page.
    const isZoomIn  = (e.ctrlKey || e.metaKey) && (e.key === '=' || e.key === '+');
    const isZoomOut = (e.ctrlKey || e.metaKey) && e.key === '-';
    const isZoomReset = (e.ctrlKey || e.metaKey) && e.key === '0';
    if (isZoomIn || isZoomOut || isZoomReset) {
      e.preventDefault();
      e.stopPropagation();
      const term = getTerm();
      const fitAddon = getFitAddon();
      if (!term) return;
      let size = term.options.fontSize;
      if (isZoomReset) {
        size = 24;
      } else if (isZoomIn) {
        size = Math.min(size + 2, 48);
      } else {
        size = Math.max(size - 2, 10);
      }
      term.options.fontSize = size;
      if (fitAddon) {
        fitAddon.fit();
        const ws = getWs();
        if (ws && ws.readyState === WebSocket.OPEN) {
          sendResize(ws, term.cols, term.rows);
        }
      }
    }
  }, true);
}

function mount(container, options) {
  const wsUrl = options.wsUrl;

  Object.assign(container.style, {
    width: '100%',
    height: '100%',
    overflow: 'hidden',
    background: '#0d1117',
  });

  let ws = null;
  let term = null;
  let fitAddon = null;
  installKeyInterception(() => ws, () => term, () => fitAddon);

  term = new Terminal({
    fontFamily: 'monospace',
    fontSize: 24,
    lineHeight: 1.0,
    scrollback: 0,
    scrollbar: { showScrollbar: false },
    theme: {
      background: '#0d1117',
      foreground: '#e6edf3',
      cursor: '#58a6ff',
    },
    allowProposedApi: true,
  });

  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  term.open(container);
  fitAddon.fit();

  const cols = term.cols || 220;
  const rows = term.rows || 50;
  const sep = wsUrl.includes('?') ? '&' : '?';
  ws = new WebSocket(wsUrl + sep + 'cols=' + cols + '&rows=' + rows);
  ws.binaryType = 'arraybuffer';

  ws.onopen = function () {
    console.log('[2wee-terminal] v' + VERSION + ' connected');
    sendResize(ws, term.cols, term.rows);
  };

  ws.onmessage = function (e) {
    if (e.data instanceof ArrayBuffer) {
      term.write(new Uint8Array(e.data));
    }
  };

  ws.onclose = function () {
    if (options.onExit) {
      window.location.href = options.onExit;
    } else {
      term.write('\r\n\x1b[2m[session ended — reload to reconnect]\x1b[0m\r\n');
    }
  };

  ws.onerror = function () {
    term.write('\r\n\x1b[31mConnection error.\x1b[0m\r\n');
  };

  term.onData(function (data) {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(new TextEncoder().encode(data));
    }
  });

  const resizeObserver = new ResizeObserver(function () {
    fitAddon.fit();
    if (ws.readyState === WebSocket.OPEN) {
      sendResize(ws, term.cols, term.rows);
    }
  });
  resizeObserver.observe(container);

  term.focus();
}

function sendResize(ws, cols, rows) {
  ws.send(JSON.stringify({ type: 'resize', cols: Math.min(cols, 500), rows: Math.min(rows, 200) }));
}

// Web component
// Attributes:
//   ws     — URL of the two_wee_terminal service (e.g. "http://localhost:7681" or "" for same host)
//   server — 2Wee Laravel server URL passed to two_wee_client (e.g. "https://myapp.com/terminal")
if (typeof customElements !== 'undefined') {
  class TwoWeeTerminalElement extends HTMLElement {
    connectedCallback() {
      const wsBase = this.getAttribute('ws') || (location.protocol + '//' + location.host);
      const server = this.getAttribute('server') || '';
      const onExit = this.getAttribute('onexit') || null;
      const wsProtocol = wsBase.startsWith('https://') ? 'wss://' : 'ws://';
      const wsHost = wsBase.replace(/^https?:\/\//, '');
      const wsUrl = wsProtocol + wsHost + '/ws' + (server ? '?server=' + encodeURIComponent(server) : '');
      Object.assign(this.style, { display: 'block', width: '100%', height: '100%' });
      mount(this, { wsUrl, onExit });
    }
  }
  customElements.define('twowee-terminal', TwoWeeTerminalElement);
}

window.TwoWeeTerminal = { mount };
