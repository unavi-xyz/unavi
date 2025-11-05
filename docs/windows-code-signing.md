# Windows Code Signing with Self-Signed Certificate

This document describes how to set up code signing for Windows executables using a self-signed certificate.

## Overview

UNAVI uses code signing for Windows executables to:

- Reduce "Unknown Publisher" warnings when users run the application
- Provide basic identity verification
- Enable SmartScreen reputation building over time

We use a self-signed certificate stored as GitHub secrets for automated signing in CI.

## Creating a Self-Signed Certificate

### Prerequisites

- Windows machine with PowerShell 5.1 or later
- Administrator privileges

### Generate the Certificate

Run the following PowerShell command as Administrator:

```powershell
$cert = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject "CN=UNAVI, O=UNAVI, C=US" `
    -KeyUsage DigitalSignature `
    -FriendlyName "UNAVI Code Signing Certificate" `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}") `
    -KeyExportPolicy Exportable `
    -KeyLength 2048 `
    -KeyAlgorithm RSA `
    -HashAlgorithm SHA256 `
    -NotAfter (Get-Date).AddYears(3)

Write-Host "Certificate Thumbprint: $($cert.Thumbprint)"
```

This creates a certificate valid for 3 years with:

- RSA 2048-bit key
- SHA256 hashing algorithm
- Code signing extended key usage
- Exportable private key

### Export the Certificate

#### Export with Private Key (PFX)

This file will be stored as a GitHub secret:

```powershell
$pfxPassword = ConvertTo-SecureString -String "YourSecurePassword123!" -Force -AsPlainText
Export-PfxCertificate `
    -Cert "Cert:\CurrentUser\My\$($cert.Thumbprint)" `
    -FilePath "unavi-codesign.pfx" `
    -Password $pfxPassword
```

**Important:** Use a strong password and store it securely.

#### Export Public Certificate (CER)

This file will be distributed with releases for users to install:

```powershell
Export-Certificate `
    -Cert "Cert:\CurrentUser\My\$($cert.Thumbprint)" `
    -FilePath "unavi-codesign.cer"
```

### Encode for GitHub Secrets

Convert the PFX file to base64 for storage as a GitHub secret:

```powershell
$bytes = [System.IO.File]::ReadAllBytes("unavi-codesign.pfx")
$base64 = [System.Convert]::ToBase64String($bytes)
$base64 | Out-File "unavi-codesign.pfx.base64.txt"
```

## Configuring GitHub Secrets

Add the following secrets to the GitHub repository:

1. **WINDOWS_CERTIFICATE**

   - Value: Contents of `unavi-codesign.pfx.base64.txt`
   - This is the base64-encoded PFX certificate with private key

1. **WINDOWS_CERTIFICATE_PASSWORD**

   - Value: The password you used when exporting the PFX
   - Example: `YourSecurePassword123!`

### Adding Secrets

1. Go to repository Settings → Secrets and variables → Actions
1. Click "New repository secret"
1. Add each secret with the name and value above

## How Signing Works in CI

The `.github/workflows/build-windows.yml` workflow automatically signs executables:

1. **Decode Certificate**: Base64-encoded PFX is decoded and written to a temporary file
1. **Sign Executable**: `signtool.exe` signs the `.exe` file with SHA256 timestamp
1. **Sign MSI**: MSI installer is signed after creation (for launcher)
1. **Cleanup**: Certificate file is securely deleted

### Signing Command

The workflow uses `signtool.exe` from Windows SDK:

```powershell
signtool sign /f certificate.pfx /p "password" /tr http://timestamp.digicert.com /td sha256 /fd sha256 executable.exe
```

Parameters:

- `/f`: Certificate file path
- `/p`: Certificate password
- `/tr`: RFC 3161 timestamp server URL
- `/td`: Timestamp digest algorithm (SHA256)
- `/fd`: File digest algorithm (SHA256)

## Installing the Certificate (For Users)

Users who download pre-releases or want to verify the signature should install the public certificate:

### Installation Steps

1. Download `unavi-codesign.cer` from the release
1. Right-click the file and select "Install Certificate"
1. Choose "Current User" as store location
1. Select "Place all certificates in the following store"
1. Click "Browse" and select "Trusted Publishers"
1. Complete the wizard

### Verification

After installation, users can verify signed executables:

```powershell
Get-AuthenticodeSignature .\unavi-launcher.exe
```

Should show:

- Status: Valid
- SignerCertificate: CN=UNAVI, O=UNAVI, C=US

## Limitations of Self-Signed Certificates

Self-signed certificates have some limitations:

1. **Not Trusted by Default**: Users must manually install the certificate
1. **No Chain of Trust**: Not issued by a recognized Certificate Authority
1. **SmartScreen Warnings**: May still show warnings until reputation is built
1. **Manual Distribution**: Certificate must be provided separately

## Troubleshooting

### Certificate Not Found

Ensure the certificate is in the correct store:

```powershell
Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert
```

### Signing Fails

Check certificate validity:

```powershell
$cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object {$_.Subject -like "*UNAVI*"}
$cert | Format-List *
```

Ensure:

- Certificate has not expired
- Private key is present (`HasPrivateKey: True`)
- Key usage includes code signing

### Timestamp Server Timeout

The workflow uses DigiCert's timestamp server. If it's unavailable, alternatives:

- `http://timestamp.comodoca.com/`
- `http://timestamp.sectigo.com/`
- `http://timestamp.globalsign.com/tsa/r6advanced1`

## Security Considerations

1. **Protect the PFX File**: The PFX contains the private key. Never commit it to the repository.
1. **Strong Password**: Use a complex password for the PFX file.
1. **Rotate Certificates**: Generate new certificates before expiration (3 year validity).
1. **Limit Access**: Only repository administrators should have access to the GitHub secrets.
1. **Audit Signing**: Review signed releases to ensure only authorized builds are signed.

## References

- [Microsoft: SignTool Documentation](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool)
- [Microsoft: Creating Self-Signed Certificates](https://learn.microsoft.com/en-us/powershell/module/pki/new-selfsignedcertificate)
- [GitHub Actions: Encrypted Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
