<#
Generate an SSH keypair for GitHub Actions deploy and print the public key.
Usage (PowerShell):
  .\generate_ssh_key.ps1 -KeyName id_rsa_plants
#>

param(
  [string]$KeyName = "id_rsa_plants"
)

if (Test-Path $KeyName -PathType Any -or Test-Path "$KeyName.pub") {
  Write-Error "Key files $KeyName or $KeyName.pub already exist. Aborting."
  exit 1
}

ssh-keygen -t rsa -b 4096 -f $KeyName -N "" -C "plants-love-rust-deploy"

Write-Host "Private key: $KeyName"
Write-Host "Public key:  $KeyName.pub"
Write-Host "Add the contents of $KeyName.pub to /home/<pi-user>/.ssh/authorized_keys on the Pi (hostname: plants-love-rust)."
Write-Host "Then add the private key contents to the GitHub repository secret named SSH_PRIVATE_KEY."
