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
        padding: 40px 50px;
        text-align: center;
        position: relative;
    }
    h1 {
        margin: 0 0 40px 0;
        color: #fff;
        font-size: 32px;
        font-weight: 600;
        letter-spacing: -0.02em;
        min-height: 38px;
    }
    .play-button {
        width: 100%;
        max-width: 300px;
        margin: 0 auto 20px;
        background: #fff;
        color: #000;
        border: none;
        border-radius: 8px;
        padding: 16px 32px;
        font-size: 18px;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.2s ease;
    }
    .play-button:hover:not(:disabled) {
        background: #e0e0e0;
    }
    .play-button:active:not(:disabled) {
        background: #d0d0d0;
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
        min-height: 20px;
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
        transition: background 0.2s ease;
        font-family: inherit;
    }
    button:hover:not(:disabled) {
        background: #e0e0e0;
    }
    button:active:not(:disabled) {
        background: #d0d0d0;
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
        margin: 0 0 20px 0;
        font-size: 13px;
    }
    .version {
        color: #444;
        font-size: 11px;
        margin-top: 20px;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }
    .gear-button {
        position: absolute;
        top: 8px;
        right: 8px;
        width: 40px;
        height: 40px;
        padding: 0;
        background: transparent !important;
        border: 1px solid transparent;
        border-radius: 50%;
        color: #666;
        font-size: 18px;
        cursor: pointer;
        transition: all 0.2s ease;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    .gear-button:hover {
        color: #aaa !important;
        background: rgba(255, 255, 255, 0.03) !important;
        border-color: rgba(255, 255, 255, 0.1);
    }
    .gear-button:active {
        color: #ccc !important;
        background: rgba(255, 255, 255, 0.05) !important;
        border-color: rgba(255, 255, 255, 0.15);
        transform: scale(0.95);
    }
"#;
