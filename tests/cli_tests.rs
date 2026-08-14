//! Testes do parse de CLI.

use std::path::PathBuf;

use qara_desktop::cli;

#[test]
fn sem_argumentos_usa_padroes() {
    let options = cli::parse(Vec::<String>::new()).unwrap();
    assert_eq!(options, cli::Options::default());
}

#[test]
fn todas_as_flags_sao_reconhecidas() {
    let options = cli::parse(vec![
        "--windowed",
        "--devtools",
        "--ui-dir",
        "/tmp/ui",
        "--check-config",
        "--version",
    ])
    .unwrap();
    assert!(options.windowed);
    assert!(options.devtools);
    assert_eq!(options.ui_dir, Some(PathBuf::from("/tmp/ui")));
    assert!(options.check_config);
    assert!(options.version);
}

#[test]
fn ui_dir_sem_valor_e_erro() {
    let err = cli::parse(vec!["--ui-dir"]).unwrap_err();
    assert!(err.contains("--ui-dir"), "{err}");
}

#[test]
fn ui_dir_seguido_de_flag_e_erro() {
    cli::parse(vec!["--ui-dir", "--windowed"]).unwrap_err();
}

#[test]
fn argumento_desconhecido_e_erro() {
    let err = cli::parse(vec!["--fullscreen"]).unwrap_err();
    assert!(err.contains("--fullscreen"), "{err}");
}
