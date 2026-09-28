#!/usr/bin/env bash
#
# Sign a macOS binary with a Developer ID Application certificate (hardened
# runtime) and notarize it with Apple. Used by .github/workflows/release.yml;
# runs locally too. The same steps and secret names as dealer3's release
# workflow, so the same values serve both repos.
#
#   scripts/macos-sign.sh target/aarch64-apple-darwin/release/rbb
#
# Environment (all optional; what is missing is skipped with a notice, so
# forks and pull requests build unsigned):
#
#   DEVELOPER_ID_CERT_BASE64    the Developer ID Application certificate and
#                               key, exported from Keychain Access as .p12,
#                               base64-encoded (base64 -i cert.p12 | pbcopy)
#   DEVELOPER_ID_CERT_PASSWORD  the .p12's export password
#   APPLE_ID                    the Apple ID that notarizes
#   APPLE_ID_PASSWORD           an app-specific password for it
#                               (appleid.apple.com, Sign-In and Security)
#   APPLE_TEAM_ID               the ten-character team ID
#   SIGN_IDENTITY               default "Developer ID Application" (any
#                               matching identity in the keychain)
#
#   SIGN_LOCAL                  set to 1 to sign with the identity already in
#                               your login keychain (DEVELOPER_ID_CERT_BASE64
#                               unset), e.g. on your own Mac
#
# A bare Mach-O binary cannot be stapled; Gatekeeper checks the notarization
# ticket online on first run. The zip submitted is the one to ship if you
# want the ticket fetched before then.

set -euo pipefail

BINARY=${1:?usage: macos-sign.sh <binary>}
IDENTITY=${SIGN_IDENTITY:-Developer ID Application}

if [[ $(uname) != Darwin ]]; then
    echo "macos-sign: not on macOS; nothing to do" >&2
    exit 0
fi

if [[ -n ${DEVELOPER_ID_CERT_BASE64:-} ]]; then
    # Import the certificate into a temporary keychain.
    KEYCHAIN=build.keychain
    KEYCHAIN_PASSWORD=$(uuidgen)
    CERT=$(mktemp -t cert).p12
    trap 'rm -f "$CERT"' EXIT
    echo "$DEVELOPER_ID_CERT_BASE64" | base64 --decode > "$CERT"
    security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
    security default-keychain -s "$KEYCHAIN"
    security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
    security import "$CERT" -k "$KEYCHAIN" \
        -P "${DEVELOPER_ID_CERT_PASSWORD:-}" -T /usr/bin/codesign
    security set-key-partition-list -S apple-tool:,apple:,codesign: \
        -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN" >/dev/null
elif [[ -z ${SIGN_LOCAL:-} ]]; then
    echo "::notice::macos-sign: DEVELOPER_ID_CERT_BASE64 not set; $BINARY left unsigned" >&2
    exit 0
fi

# Hardened runtime: required for notarization.
codesign --force --options runtime --timestamp --sign "$IDENTITY" "$BINARY"
codesign --verify --verbose "$BINARY"

if [[ -z ${APPLE_ID:-} || -z ${APPLE_ID_PASSWORD:-} || -z ${APPLE_TEAM_ID:-} ]]; then
    echo "::notice::macos-sign: APPLE_ID, APPLE_ID_PASSWORD or APPLE_TEAM_ID not set; signed, not notarized" >&2
    exit 0
fi

# Notarization takes a zip.
ZIP=$(mktemp -d)/notarize.zip
ditto -c -k --keepParent "$BINARY" "$ZIP"
xcrun notarytool submit "$ZIP" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_ID_PASSWORD" \
    --team-id "$APPLE_TEAM_ID" \
    --wait
rm -f "$ZIP"
echo "macos-sign: $BINARY signed and notarized"
