//! Parse dos argumentos de linha de comando (sem crate clap — 5 flags
//! simples não justificam a dependência).

use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// Janela normal única (debug), sem layer-shell.
    pub windowed: bool,
    /// Habilita DevTools do WebKit (desabilitado por padrão).
    pub devtools: bool,
    /// Carrega a UI do disco em vez dos assets embutidos (desenvolvimento).
    pub ui_dir: Option<PathBuf>,
    /// Valida a config e sai (código 0/1).
    pub check_config: bool,
    /// Imprime a versão e sai.
    pub version: bool,
}

pub const USAGE: &str =
    "uso: qara-desktop [--windowed] [--devtools] [--ui-dir <dir>] [--check-config] [--version]";

/// Faz o parse de `args` (sem o argv[0]). Erro é mensagem pt-BR pronta
/// para o stderr.
pub fn parse<I, S>(args: I) -> Result<Options, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut options = Options::default();
    let mut iter = args.into_iter().map(Into::into);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--windowed" => options.windowed = true,
            "--devtools" => options.devtools = true,
            "--check-config" => options.check_config = true,
            "--version" => options.version = true,
            "--ui-dir" => match iter.next() {
                Some(dir) if !dir.starts_with("--") => options.ui_dir = Some(PathBuf::from(dir)),
                _ => return Err(format!("--ui-dir exige um diretório.\n{USAGE}")),
            },
            other => return Err(format!("argumento desconhecido: {other}\n{USAGE}")),
        }
    }
    Ok(options)
}
