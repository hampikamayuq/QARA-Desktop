//! Binário do QARA Desktop: parse de CLI, logging e handoff para `app`.

use std::process::ExitCode;

use qara_desktop::{cli, config, logging, paths};

fn main() -> ExitCode {
    let options = match cli::parse(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("qara-desktop: {message}");
            return ExitCode::from(2);
        }
    };

    if options.version {
        println!("qara-desktop {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    if options.check_config {
        return check_config();
    }

    let _log_guard = logging::init(&paths::state_dir());
    logging::install_panic_hook();
    tracing::info!(
        versao = env!("CARGO_PKG_VERSION"),
        opcoes = ?options,
        "iniciando QARA Desktop"
    );
    logging::log_environment();

    let status = qara_desktop::app::run(options);
    tracing::info!(status, "encerrando QARA Desktop");
    ExitCode::from(u8::try_from(status).unwrap_or(1))
}

/// `--check-config`: valida a config do operador e sai com 0/1.
/// Saída pensada para operação (sem depender do arquivo de log).
fn check_config() -> ExitCode {
    let path = paths::config_file();
    match std::fs::read_to_string(&path) {
        Ok(raw) => match config::parse_and_validate(&raw) {
            Ok(config) => {
                println!(
                    "OK: {} válida (perfil \"{}\", {} grupo(s))",
                    path.display(),
                    config.profile,
                    config.groups.len()
                );
                ExitCode::SUCCESS
            }
            Err(problems) => {
                eprintln!("ERRO: {} inválida:\n{problems}", path.display());
                ExitCode::FAILURE
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "OK: {} ausente; o fallback embutido será usado",
                path.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("ERRO: falha ao ler {}: {e}", path.display());
            ExitCode::FAILURE
        }
    }
}
