pub const APP_STYLES: &str = r#"
    * {
        box-sizing: border-box;
    }
    body {
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Oxygen", "Ubuntu", "Cantarell", sans-serif;
        background: #0d0d0d;
        margin: 0;
        padding: 0;
        color: #f5f5f5;
    }
    .container {
        background: #0d0d0d;
        border: 1px solid #2a2a2a;
        border-radius: 8px;
        padding: 32px;
        margin: 32px;
    }
    h1 {
        margin: 0 0 24px 0;
        color: #f5f5f5;
        font-size: 18px;
        font-weight: 500;
        letter-spacing: -0.01em;
    }
    .status {
        margin: 16px 0;
        padding: 0;
        color: #a0a0a0;
        font-size: 14px;
        line-height: 1.6;
    }
    .loading {
        display: inline-block;
        width: 14px;
        height: 14px;
        border: 2px solid #2a2a2a;
        border-top: 2px solid #f5f5f5;
        border-radius: 50%;
        animation: spin 1s cubic-bezier(0.4, 0, 0.2, 1) infinite;
        margin-right: 10px;
        vertical-align: middle;
    }
    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }
    button {
        background: #f5f5f5;
        color: #0d0d0d;
        border: 1px solid #f5f5f5;
        border-radius: 6px;
        padding: 12px 20px;
        font-size: 14px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        font-family: inherit;
        width: 100%;
    }
    button:hover {
        background: #0d0d0d;
        color: #f5f5f5;
        transform: translateY(-1px);
    }
    button:active {
        transform: translateY(0);
    }
    button:disabled {
        background: #2a2a2a;
        border-color: #2a2a2a;
        color: #666;
        cursor: not-allowed;
        transform: none;
    }
    .error {
        background: #1a1a1a;
        border: 1px solid #f5f5f5;
        border-radius: 6px;
        color: #f5f5f5;
        padding: 14px;
        margin: 16px 0;
        font-size: 13px;
        line-height: 1.5;
    }
    .version {
        color: #666;
        font-size: 12px;
        margin-top: 24px;
    }
"#;
