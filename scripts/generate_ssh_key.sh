#!/usr/bin/env bash
# Generate an SSH keypair for GitHub Actions deploy and print the public key.
# Usage: ./generate_ssh_key.sh id_rsa_plants

set -euo pipefail

KEY_NAME=${1:-id_rsa_plants}

if [ -e "$KEY_NAME" ] || [ -e "$KEY_NAME.pub" ]; then
  echo "Key files $KEY_NAME or $KEY_NAME.pub already exist. Aborting." >&2
  exit 1
fi

ssh-keygen -t rsa -b 4096 -f "$KEY_NAME" -N "" -C "plants-love-rust-deploy"

echo
echo "Private key: $KEY_NAME"
echo "Public key:  $KEY_NAME.pub"
echo
echo "Add the contents of $KEY_NAME.pub to /home/<pi-user>/.ssh/authorized_keys on the Pi (hostname: plants-love-rust)."
echo "Then add the private key contents to the GitHub repository secret named SSH_PRIVATE_KEY."
