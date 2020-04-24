use std::collections::HashMap;
use std::fs::File;
use std::fs::Permissions;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

use failure::ResultExt;
use structopt::StructOpt;
use tokio::process::Command;

use crate::conf::Conf;
use crate::ErrorKind;
use crate::Result;

/// Configuration related commands.
#[derive(Debug, StructOpt)]
pub struct CliOpt {
    /// Regenerate certificates even if they exist.
    #[structopt(name = "regen", long)]
    regenerate: bool,
}

/// Configuration related commands.
pub async fn run(args: CliOpt, conf: Conf) -> Result<bool> {
    if !conf.project.allow_gen_certs() {
        let error = ErrorKind::invalid_project(conf.project, "replidev gen-certs");
        return Err(error.into());
    }

    // Check if PKI certs exist.
    let pki_path = crate::settings::Paths::pki(&conf.project);
    let ca_cert = format!("{}/replidev/certs/replidev.crt", pki_path);
    let pki_found = std::path::Path::new(&ca_cert).exists();
    if !args.regenerate && pki_found {
        println!("Certificates already available at {}", pki_path);
        println!("To regenerate the certificates invoke the command with --regen");
        return Ok(true);
    }
    if args.regenerate && pki_found {
        std::fs::remove_dir_all(format!("{}/replidev", pki_path))
            .with_context(|_| ErrorKind::fs_not_allowed(pki_path))?;
    }

    // Setup PKI path if needed.
    if !std::path::Path::new(pki_path).exists() {
        std::fs::create_dir_all(pki_path).with_context(|_| ErrorKind::fs_not_allowed(pki_path))?;
    }

    // Generate the certificates.
    let easypki = &conf.easypki;
    let env = {
        let mut env = HashMap::new();
        env.insert("PKI_ROOT", pki_path);
        env.insert("PKI_ORGANIZATION", "Replicante Development PKI");
        env.insert("PKI_ORGANIZATIONAL_UNIT", "Development");
        env.insert("PKI_COUNTRY", "EU");
        env
    };

    println!("--> Generating CA certificates");
    let status = Command::new(easypki)
        .envs(env.iter())
        .arg("create")
        .arg("--private-key-size=4096")
        .arg("--ca")
        .arg("replidev")
        .status()
        .await
        .with_context(|_| ErrorKind::command_exec("easypki create ca"))?;
    if !status.success() {
        let error = ErrorKind::command_failed("easypki create ca");
        return Err(error.into());
    }

    println!("--> Generating Server certificates");
    let status = Command::new(easypki)
        .envs(env.iter())
        .arg("create")
        .arg("--private-key-size=4096")
        .arg("--ca-name=replidev")
        .arg("--dns=localhost")
        .arg("server")
        .status()
        .await
        .with_context(|_| ErrorKind::command_exec("easypki create server"))?;
    if !status.success() {
        let error = ErrorKind::command_failed("easypki create server");
        return Err(error.into());
    }

    println!("--> Generating Client certificates");
    let status = Command::new(easypki)
        .envs(env.iter())
        .arg("create")
        .arg("--private-key-size=4096")
        .arg("--ca-name=replidev")
        .arg("--client")
        .arg("client")
        .status()
        .await
        .with_context(|_| ErrorKind::command_exec("easypki create client"))?;
    if !status.success() {
        let error = ErrorKind::command_failed("easypki create client");
        return Err(error.into());
    }

    // Some tools require certificate and key in one file.
    // Create some bundles for these applications.
    println!("--> Bundling some certs");
    let bundles_path = format!("{}/replidev/bundles", pki_path);
    if !std::path::Path::new(&bundles_path).exists() {
        std::fs::create_dir_all(&bundles_path)
            .with_context(|_| ErrorKind::fs_not_allowed(bundles_path))?;
    }
    bundle_certs(pki_path, "client")?;
    bundle_certs(pki_path, "server")?;

    // Update keys path permissions to allow non-root pods to access them.
    let perms = Permissions::from_mode(0o755);
    std::fs::set_permissions(format!("{}/replidev/keys/", pki_path), perms)
        .with_context(|_| ErrorKind::fs_error("unable to change permissions"))?;

    // Print all certificate paths for user to know.
    println!(
        "CA Certificate:     {}/replidev/certs/replidev.crt",
        pki_path
    );
    println!(
        "CA Private Key:     {}/replidev/keys/replidev.key",
        pki_path
    );
    println!(
        "Client Bundle:      {}/replidev/bundles/client.pem",
        pki_path
    );
    println!("Client Certificate: {}/replidev/certs/client.crt", pki_path);
    println!("Client Private Key: {}/replidev/keys/client.key", pki_path);
    println!(
        "Server Bundle:      {}/replidev/bundles/server.pem",
        pki_path
    );
    println!("Server Certificate: {}/replidev/certs/server.crt", pki_path);
    println!("Server Private Key: {}/replidev/keys/server.key", pki_path);
    Ok(true)
}

fn bundle_certs(pki_path: &str, what: &str) -> Result<()> {
    let cert_path = format!("{}/replidev/certs/{}.crt", pki_path, what);
    let key_path = format!("{}/replidev/keys/{}.key", pki_path, what);
    let bundle_path = format!("{}/replidev/bundles/{}.pem", pki_path, what);
    let mut bundle = Vec::new();
    let mut cert =
        File::open(&cert_path).with_context(|_| ErrorKind::fs_not_allowed(&cert_path))?;
    cert.read_to_end(&mut bundle)
        .with_context(|_| ErrorKind::fs_error(&format!("unable to read {}", cert_path)))?;
    let mut key = File::open(&key_path).with_context(|_| ErrorKind::fs_not_allowed(&key_path))?;
    key.read_to_end(&mut bundle)
        .with_context(|_| ErrorKind::fs_error(&format!("unable to read {}", key_path)))?;
    let mut bundle_file =
        File::create(&bundle_path).with_context(|_| ErrorKind::fs_not_allowed(&bundle_path))?;
    bundle_file
        .write_all(&bundle)
        .with_context(|_| ErrorKind::fs_error(&format!("unable to write to {}", bundle_path)))?;
    Ok(())
}
