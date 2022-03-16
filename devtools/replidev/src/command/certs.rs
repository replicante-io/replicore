use std::collections::HashMap;
use std::fs::File;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

use anyhow::Context;
use anyhow::Result;
use structopt::StructOpt;
use tokio::process::Command;

use crate::conf::Conf;
use crate::error::InvalidProject;

/// Configuration related commands.
#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Regenerate certificates even if they exist.
    #[structopt(name = "regen", long)]
    regenerate: bool,
}

pub async fn run(args: Opt, conf: Conf) -> Result<i32> {
    if !conf.project.allow_gen_certs() {
        anyhow::bail!(InvalidProject::new(conf.project, "gen-certs"));
    }

    // Compute location for PKI files.
    let pki_path = <dyn crate::settings::Paths>::pki(&conf.project);
    let pki_path = &format!("{}/replidev", pki_path);

    // Check if PKI certs exist.
    let ca_cert = &format!("{}/ca.crt", pki_path);
    let pki_found = std::path::Path::new(ca_cert).exists();
    if !args.regenerate && pki_found {
        println!("Certificates already available at {}", pki_path);
        println!("To regenerate the certificates invoke the command with --regen.\n");
        print_certificate_locations(pki_path);
        return Ok(0);
    }

    // Wipe PKI tree if user requested a regeneration.
    if args.regenerate && pki_found {
        std::fs::remove_dir_all(pki_path)
            .with_context(|| format!("Unable to clear PKI data tree at {}", pki_path))?;
    }

    // Initialise the PKI tree.
    if !std::path::Path::new(pki_path).exists() {
        std::fs::create_dir_all(pki_path)
            .with_context(|| format!("Unable to create PKI data tree at {}", pki_path))?;
    }

    // Get the commands you need.
    let easyrsa = &conf.easyrsa;
    let openssl = &conf.openssl;

    // Prepare the easyrsa environment.
    let env = {
        let mut env = HashMap::new();
        env.insert("EASYRSA_BATCH", "1");
        env.insert("EASYRSA_CA_EXPIRE", "365");
        env.insert("EASYRSA_CERT_EXPIRE", "365");
        env.insert("EASYRSA_KEY_SIZE", "4096");
        env.insert("EASYRSA_OPENSSL", openssl);
        env.insert("EASYRSA_PKI", pki_path);
        env.insert("EASYRSA_REQ_COUNTRY", "EU");
        env.insert("EASYRSA_REQ_ORG", "Replicante Development PKI");
        env.insert("EASYRSA_REQ_OU", "Development");
        env
    };

    // Initialise the Certificate Authority.
    println!("--> Initialising Certificate Authotity");
    let status = Command::new(easyrsa)
        .envs(env.iter())
        .arg("init-pki")
        .status()
        .await
        .context("unable to initialise the EasyRSA PKI")?;
    if !status.success() {
        let code = match status.code() {
            None => "N/A".to_string(),
            Some(code) => code.to_string(),
        };
        anyhow::bail!("EasyRSA PKI initialisation failed with code {}", code);
    }

    // Create the Certificate Authority.
    println!("--> Creating Certificate Authotity");
    let status = Command::new(easyrsa)
        .envs(env.iter())
        .arg("build-ca")
        .arg("nopass") // Don't ask for a Passphrase on the dev CA.
        .status()
        .await
        .context("unable to build the EasyRSA PKI CA")?;
    if !status.success() {
        let code = match status.code() {
            None => "N/A".to_string(),
            Some(code) => code.to_string(),
        };
        anyhow::bail!("EasyRSA CA build failed with code {}", code);
    }

    // Create the Server Certificate.
    println!("--> Generating Server certificates");
    let ip = conf
        .podman_host_ip()
        .map_err(crate::error::wrap_for_anyhow)
        .context("Unable to detect local IP for certificate alternative names")?;
    let status = Command::new(easyrsa)
        .envs(env.iter())
        .arg("--req-cn=server")
        .arg("gen-req")
        .arg("server")
        .arg("nopass") // Don't ask for a Passphrase on the dev CA.
        .status()
        .await
        .context("unable to generate EasyRSA server request")?;
    if !status.success() {
        let code = match status.code() {
            None => "N/A".to_string(),
            Some(code) => code.to_string(),
        };
        anyhow::bail!("EasyRSA server request failed with code {}", code);
    }
    let status = Command::new(easyrsa)
        .envs(env.iter())
        .arg("--req-cn=server")
        .arg(format!(
            "--subject-alt-name=DNS:localhost,DNS:podlan-host,DNS:host.containers.internal,IP:{}",
            ip,
        ))
        .arg("sign-req")
        .arg("server") // Certificate type.
        .arg("server") // Certificate name.
        .arg("nopass") // Don't ask for a Passphrase on the dev CA.
        .status()
        .await
        .context("unable to sign EasyRSA server certificate")?;
    if !status.success() {
        let code = match status.code() {
            None => "N/A".to_string(),
            Some(code) => code.to_string(),
        };
        anyhow::bail!("EasyRSA server signing failed with code {}", code);
    }

    // Creare the Client Certificate.
    let status = Command::new(easyrsa)
        .envs(env.iter())
        .arg("--req-cn=client")
        .arg("gen-req")
        .arg("client")
        .arg("nopass") // Don't ask for a Passphrase on the dev CA.
        .status()
        .await
        .context("unable to generate EasyRSA client request")?;
    if !status.success() {
        let code = match status.code() {
            None => "N/A".to_string(),
            Some(code) => code.to_string(),
        };
        anyhow::bail!("EasyRSA client request failed with code {}", code);
    }
    let status = Command::new(easyrsa)
        .envs(env.iter())
        .arg("--req-cn=client")
        .arg("sign-req")
        .arg("client") // Certificate type.
        .arg("client") // Certificate name.
        .arg("nopass") // Don't ask for a Passphrase on the dev CA.
        .status()
        .await
        .context("unable to sign EasyRSA client certificate")?;
    if !status.success() {
        let code = match status.code() {
            None => "N/A".to_string(),
            Some(code) => code.to_string(),
        };
        anyhow::bail!("EasyRSA client signing failed with code {}", code);
    }

    // Bundle certificates up for clients that need them in one file.
    // Some tools require certificate and key in one file so we provide that too.
    println!("--> Bundling private and public keys for certificates");
    let bundles_path = format!("{}/bundles", pki_path);
    if !std::path::Path::new(&bundles_path).exists() {
        std::fs::create_dir_all(&bundles_path)
            .with_context(|| format!("Unable to create bundles path: {}", bundles_path))?;
    }
    bundle_keypair_into_pem(openssl, pki_path, "client").await?;
    bundle_keypair_into_pem(openssl, pki_path, "server").await?;

    // Update keys path permissions to allow non-root pods to access them.
    println!("--> Updating file system permissions for dev access");
    update_fs_permissions(format!("{}/bundles", pki_path), 0o755)?;
    update_fs_permissions(format!("{}/issued", pki_path), 0o755)?;
    update_fs_permissions(format!("{}/private", pki_path), 0o755)?;
    update_fs_permissions(format!("{}/ca.crt", pki_path), 0o644)?;
    update_fs_permissions(format!("{}/private/ca.key", pki_path), 0o644)?;
    update_fs_permissions(format!("{}/bundles/client.pem", pki_path), 0o644)?;
    update_fs_permissions(format!("{}/bundles/server.pem", pki_path), 0o644)?;
    update_fs_permissions(format!("{}/issued/client.crt", pki_path), 0o644)?;
    update_fs_permissions(format!("{}/issued/server.crt", pki_path), 0o644)?;
    update_fs_permissions(format!("{}/private/client.key", pki_path), 0o644)?;
    update_fs_permissions(format!("{}/private/server.key", pki_path), 0o644)?;

    // Done.
    print_certificate_locations(pki_path);
    Ok(0)
}

/// Bundle public and private key for a certificate into a combined PEM file.
async fn bundle_keypair_into_pem(openssl: &str, pki_path: &str, what: &str) -> Result<()> {
    // Figure out which files we need to look at.
    let cert_path = format!("{}/issued/{}.crt", pki_path, what);
    let key_path = format!("{}/private/{}.key", pki_path, what);
    let bundle_path = format!("{}/bundles/{}.pem", pki_path, what);

    // Export the public side into the bundle.
    let bundle = File::create(&bundle_path)
        .with_context(|| format!("Unable to open {} for write", &bundle_path))?;
    let status = Command::new(openssl)
        .arg("x509")
        .arg("-in")
        .arg(cert_path)
        .stdout(bundle)
        .status()
        .await
        .with_context(|| format!("Unable to read {} public certificate with openssl", what))?;
    if !status.success() {
        let code = match status.code() {
            None => "N/A".to_string(),
            Some(code) => code.to_string(),
        };
        anyhow::bail!(
            "OpenSSL unable to read {} public certificate with code {}",
            what,
            code,
        );
    }

    // Export the private side into the bundle.
    let bundle = File::options()
        .append(true)
        .open(&bundle_path)
        .with_context(|| format!("Unable to open {} for append", &bundle_path))?;
    let status = Command::new(openssl)
        .arg("rsa")
        .arg("-in")
        .arg(key_path)
        .stdout(bundle)
        .status()
        .await
        .with_context(|| format!("Unable to read {} private key with openssl", what))?;
    if !status.success() {
        let code = match status.code() {
            None => "N/A".to_string(),
            Some(code) => code.to_string(),
        };
        anyhow::bail!(
            "OpenSSL unable to read {} private key with code {}",
            what,
            code,
        );
    }

    // Done.
    Ok(())
}

/// Print all certificate paths for user to know.
fn print_certificate_locations(pki_path: &str) {
    println!("--> Here is where you can find your certificates:");
    println!("CA Certificate:     {}/ca.crt", pki_path);
    println!("CA Private Key:     {}/private/ca.key", pki_path);
    println!("Client Bundle:      {}/bundles/client.pem", pki_path);
    println!("Client Certificate: {}/issued/client.crt", pki_path);
    println!("Client Private Key: {}/private/client.key", pki_path);
    println!("Server Bundle:      {}/bundles/server.pem", pki_path);
    println!("Server Certificate: {}/issued/server.crt", pki_path);
    println!("Server Private Key: {}/private/server.key", pki_path);
}

/// Update file system permissions for a file or directory.
fn update_fs_permissions<S>(path: S, mode: u32) -> Result<()>
where
    S: AsRef<str>,
{
    let perms = Permissions::from_mode(mode);
    std::fs::set_permissions(path.as_ref(), perms)
        .with_context(|| format!("Unable to update permissions for {}", path.as_ref()))?;
    Ok(())
}
