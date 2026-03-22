use axum::response::{IntoResponse, Response};
use axum::http::header;

const XTERM_CSS: &str = include_str!("frontend/terminal.css");
pub const TERMINAL_JS: &str = include_str!("frontend/terminal.js");

const BUILD_TIME: &str = env!("BUILD_TIME");

const LANDING_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Cache-Control" content="no-store">
  <title>2Wee Terminal</title>
  <style>
    @keyframes blink {
      0%, 49% { opacity: 0; }
      50%, 100% { opacity: 1; }
    }
    * { margin: 0; padding: 0; box-sizing: border-box; }
    html, body {
      width: 100%; height: 100vh;
      background: #1E1E2E;
      color: #E0E0E0;
      font-family: "SF Mono", "Menlo", "Cascadia Code", "Consolas", monospace;
      font-weight: 500;
      display: flex;
      align-items: center;
      justify-content: center;
      -webkit-font-smoothing: antialiased;
    }
    .container {
      width: 100%;
      max-width: 480px;
      padding: 0 24px;
    }
    .logo {
      margin-bottom: 16px;
    }
    .logo svg {
      height: 36px;
      width: auto;
    }
    .logo svg rect {
      animation: blink 1.05s step-end 5 forwards;
    }
    .tagline {
      color: #808890;
      font-size: 13px;
      margin-bottom: 40px;
    }
    label {
      display: block;
      font-size: 11px;
      color: #808890;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      margin-bottom: 8px;
    }
    .input-row {
      display: flex;
      gap: 8px;
    }
    input[type="url"] {
      flex: 1;
      background: #141420;
      border: 1px solid #3A3A5C;
      color: #E0E0E0;
      font-family: "SF Mono", "Menlo", "Cascadia Code", "Consolas", monospace;
      font-size: 13px;
      font-weight: 500;
      padding: 10px 12px;
      border-radius: 4px;
      outline: none;
    }
    input[type="url"]:focus { border-color: #56B6C2; }
    button {
      background: #2E6B73;
      color: #E0E0E0;
      font-family: "SF Mono", "Menlo", "Cascadia Code", "Consolas", monospace;
      font-size: 13px;
      font-weight: 600;
      border: 1px solid #2E6B73;
      padding: 10px 20px;
      border-radius: 4px;
      cursor: pointer;
      white-space: nowrap;
    }
    button:hover { background: #3a8a96; border-color: #56B6C2; color: #ffffff; }
    .examples {
      margin-top: 20px;
      font-size: 12px;
      color: #808890;
    }
    .examples span {
      color: #56B6C2;
      cursor: pointer;
    }
    .examples span:hover { color: #6DC4CE; text-decoration: underline; }
  </style>
</head>
<body>
  <div class="container">
    <div class="logo">
      <svg width="292" height="115" viewBox="0 0 292 115" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M5.60014 92V81.4744L30.1935 58.7028C32.2852 56.6786 34.0394 54.8569 35.4563 53.2376C36.8957 51.6183 37.9865 50.0327 38.7287 48.4808C39.4709 46.9065 39.842 45.2085 39.842 43.3867C39.842 41.3626 39.3809 39.6196 38.4588 38.1577C37.5367 36.6733 36.2772 35.5375 34.6804 34.7504C33.0836 33.9407 31.2731 33.5359 29.2489 33.5359C27.1348 33.5359 25.2906 33.9632 23.7163 34.8178C22.1419 35.6725 20.9274 36.8982 20.0728 38.495C19.2182 40.0919 18.7908 41.9923 18.7908 44.1964H4.92543C4.92543 39.6758 5.94875 35.7512 7.99538 32.4226C10.042 29.094 12.9096 26.5188 16.598 24.6971C20.2865 22.8754 24.5372 21.9645 29.3501 21.9645C34.2981 21.9645 38.605 22.8416 42.271 24.5959C45.9594 26.3276 48.8269 28.7341 50.8736 31.8153C52.9202 34.8965 53.9435 38.4276 53.9435 42.4084C53.9435 45.0173 53.4263 47.5924 52.3917 50.1339C51.3796 52.6753 49.5691 55.4979 46.9602 58.6016C44.3513 61.6828 40.6741 65.3825 35.9286 69.7006L25.8416 79.5852V80.0575H54.8544V92H5.60014Z" fill="#72CAC4"/>
        <path d="M76.646 92L56.8769 22.9091H72.8339L84.2703 70.9151H84.8438L97.461 22.9091H111.124L123.707 71.0163H124.315L135.751 22.9091H151.708L131.939 92H117.703L104.546 46.8278H104.006L90.8825 92H76.646ZM172.555 93.0121C167.225 93.0121 162.637 91.9325 158.791 89.7734C154.968 87.5919 152.021 84.5107 149.952 80.5298C147.883 76.5265 146.849 71.7923 146.849 66.3271C146.849 60.9968 147.883 56.3188 149.952 52.293C152.021 48.2672 154.934 45.1297 158.69 42.8807C162.468 40.6316 166.899 39.5071 171.982 39.5071C175.4 39.5071 178.583 40.0581 181.529 41.1602C184.498 42.2397 187.084 43.8703 189.288 46.0518C191.515 48.2334 193.247 50.9773 194.483 54.2834C195.72 57.567 196.339 61.4129 196.339 65.821V69.7681H152.584V60.8619H182.811C182.811 58.7927 182.361 56.9598 181.461 55.3629C180.562 53.7661 179.314 52.5179 177.717 51.6183C176.142 50.6961 174.309 50.2351 172.218 50.2351C170.036 50.2351 168.102 50.7411 166.415 51.7532C164.751 52.7428 163.447 54.081 162.502 55.7678C161.557 57.4321 161.074 59.2875 161.051 61.3342V69.8018C161.051 72.3658 161.524 74.5811 162.468 76.4478C163.435 78.3145 164.796 79.7539 166.55 80.766C168.305 81.7781 170.385 82.2841 172.791 82.2841C174.388 82.2841 175.85 82.0592 177.177 81.6094C178.504 81.1596 179.64 80.4848 180.584 79.5852C181.529 78.6856 182.249 77.5836 182.743 76.2791L196.035 77.1562C195.361 80.3499 193.977 83.1387 191.886 85.5227C189.817 87.8842 187.14 89.7285 183.857 91.0554C180.596 92.3598 176.828 93.0121 172.555 93.0121ZM224.583 93.0121C219.252 93.0121 214.664 91.9325 210.818 89.7734C206.995 87.5919 204.049 84.5107 201.98 80.5298C199.91 76.5265 198.876 71.7923 198.876 66.3271C198.876 60.9968 199.91 56.3188 201.98 52.293C204.049 48.2672 206.961 45.1297 210.717 42.8807C214.496 40.6316 218.926 39.5071 224.009 39.5071C227.428 39.5071 230.61 40.0581 233.556 41.1602C236.525 42.2397 239.111 43.8703 241.316 46.0518C243.542 48.2334 245.274 50.9773 246.511 54.2834C247.748 57.567 248.366 61.4129 248.366 65.821V69.7681H204.611V60.8619H234.838C234.838 58.7927 234.388 56.9598 233.489 55.3629C232.589 53.7661 231.341 52.5179 229.744 51.6183C228.17 50.6961 226.337 50.2351 224.245 50.2351C222.064 50.2351 220.129 50.7411 218.443 51.7532C216.778 52.7428 215.474 54.081 214.529 55.7678C213.585 57.4321 213.101 59.2875 213.079 61.3342V69.8018C213.079 72.3658 213.551 74.5811 214.496 76.4478C215.463 78.3145 216.823 79.7539 218.578 80.766C220.332 81.7781 222.412 82.2841 224.819 82.2841C226.416 82.2841 227.877 82.0592 229.204 81.6094C230.531 81.1596 231.667 80.4848 232.612 79.5852C233.556 78.6856 234.276 77.5836 234.771 76.2791L248.063 77.1562C247.388 80.3499 246.005 83.1387 243.913 85.5227C241.844 87.8842 239.168 89.7285 235.884 91.0554C232.623 92.3598 228.856 93.0121 224.583 93.0121Z" fill="white" fill-opacity="0.81"/>
        <rect x="287" y="6" width="5" height="106" fill="#7C8493"/>
      </svg>
    </div>
    <div class="tagline">Connect to a 2Wee server</div>
    <form id="form" onsubmit="connect(event)">
      <label for="server">Server URL</label>
      <div class="input-row">
        <input id="server" type="url" placeholder="https://myapp.example.com" required autofocus>
        <button type="submit">Connect</button>
      </div>
    </form>
    <div class="examples">
      Examples:&nbsp;
      <span onclick="fill('https://demo.2wee.dev/terminal')">demo.2wee.dev</span>
    </div>
  </div>
  <script>
    function fill(url) {
      document.getElementById('server').value = url;
      document.getElementById('server').focus();
    }
    function connect(e) {
      e.preventDefault();
      const url = document.getElementById('server').value.trim();
      if (url) window.location.href = '/?server=' + encodeURIComponent(url);
    }
    const params = new URLSearchParams(location.search);
    const pre = params.get('server');
    if (pre) document.getElementById('server').value = pre;
  </script>
</body>
</html>"##;

/// Full-screen terminal page for a known server URL.
pub async fn terminal_page(server_url: &str) -> Response {
    // Build in parts so XTERM_CSS (which contains '%') never enters a format! string.
    let html = String::new()
        + r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Cache-Control" content="no-store">
  <title>2Wee</title>
  <style>
"#
        + XTERM_CSS
        + r#"    * { margin: 0; padding: 0; box-sizing: border-box; }
    html, body { width: 100%; max-width: 100vw; height: 100vh; background: #0d1117; overflow: hidden; }
    #terminal { position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: #0d1117; }
    .xterm { width: 100% !important; height: 100% !important; }
    .xterm-screen { width: 100% !important; height: 100% !important; background: #0d1117; }
    .xterm-screen canvas { width: 100% !important; height: 100% !important; }
    .xterm-viewport { display: none !important; }
    .xterm-helpers { position: absolute !important; }
    .xterm-rows { background: #0d1117; }
  </style>
</head>
<body>
  <div id="terminal"></div>
  <script src="/terminal.js?v="#
        + BUILD_TIME
        + r#"" onload="TwoWeeTerminal.mount(document.getElementById('terminal'), { wsUrl: (location.protocol === 'https:' ? 'wss://' : 'ws://') + location.host + '/ws?server=' + encodeURIComponent('"#
        + server_url
        + r#"') })"></script>
</body>
</html>"#;

    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        html,
    )
        .into_response()
}

/// Landing page served when no server URL is provided.
pub async fn landing_page() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        LANDING_HTML,
    )
        .into_response()
}

pub async fn js_handler() -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/javascript"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        TERMINAL_JS,
    )
        .into_response()
}
