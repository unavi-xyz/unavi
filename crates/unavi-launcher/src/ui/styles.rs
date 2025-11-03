pub const APP_STYLES: &str = r#"
    * {
        box-sizing: border-box;
    }
    body {
        font-family: system-ui, -apple-system, sans-serif;
        background: #000;
        margin: 0;
        padding: 0;
        color: #fff;
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 100vh;
    }
    .container {
        width: 100%;
        max-width: 400px;
        padding: 40px;
        text-align: center;
    }
    h1 {
        margin: 0 0 40px 0;
        color: #fff;
        font-size: 32px;
        font-weight: 600;
        letter-spacing: -0.02em;
    }
    .play-button {
        width: 100%;
        max-width: 300px;
        margin: 0 auto 40px;
        background: #fff;
        color: #000;
        border: none;
        border-radius: 8px;
        padding: 16px 32px;
        font-size: 18px;
        font-weight: 600;
        cursor: pointer;
        transition: opacity 0.15s;
    }
    .play-button:hover:not(:disabled) {
        opacity: 0.9;
    }
    .play-button:disabled {
        background: #333;
        color: #666;
        cursor: not-allowed;
    }
    .settings {
        margin: 30px 0;
    }
    .settings label {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        color: #888;
        font-size: 14px;
        cursor: pointer;
    }
    .settings input[type="checkbox"] {
        cursor: pointer;
        width: 16px;
        height: 16px;
    }
    .status {
        margin: 20px 0;
        color: #888;
        font-size: 14px;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 10px;
    }
    .loading {
        width: 14px;
        height: 14px;
        border: 2px solid #333;
        border-top-color: #fff;
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }
    @keyframes spin {
        to { transform: rotate(360deg); }
    }
    button {
        background: #fff;
        color: #000;
        border: none;
        border-radius: 8px;
        padding: 12px 24px;
        font-size: 14px;
        font-weight: 500;
        cursor: pointer;
        transition: opacity 0.15s;
        font-family: inherit;
    }
    button:hover:not(:disabled) {
        opacity: 0.9;
    }
    button:disabled {
        background: #333;
        color: #666;
        cursor: not-allowed;
    }
    .error {
        background: #1a0000;
        border: 1px solid #500;
        border-radius: 8px;
        color: #f88;
        padding: 12px;
        margin: 20px 0;
        font-size: 13px;
    }
    .version {
        color: #444;
        font-size: 11px;
        margin-top: 40px;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }
    .gear-button {
        position: absolute;
        top: 20px;
        right: 20px;
        width: 32px;
        height: 32px;
        padding: 0;
        background: transparent;
        border: 1px solid #333;
        border-radius: 6px;
        color: #666;
        font-size: 16px;
        cursor: pointer;
        transition: all 0.15s;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    .gear-button:hover {
        background: #111;
        border-color: #666;
        color: #999;
    }
"#;
