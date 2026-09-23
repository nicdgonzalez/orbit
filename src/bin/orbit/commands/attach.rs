use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

use anyhow::Context as _;
use orbit::app::attach_to_session;
use orbit::session::list_sessions;
use orbit::session_id::SessionId;
use tracing::error;

use crate::commands::{Context, Run};

#[derive(clap::Args)]
pub struct Args {
    query: Option<String>,

    #[clap(long)]
    path: Option<PathBuf>,
}

impl Run for Args {
    fn run(self, _: Context) -> anyhow::Result<()> {
        if let Some(ref path) = self.path {
            _ = attach_to_session(path).context("failed to open session")?;
            return Ok(());
        }

        let directories = parse_orbit_path().context("failed to parse ORBIT_PATH")?;
        let query = self.query.as_deref().unwrap_or("");

        if let Some(path) = prompt_to_select_option(&directories, query)
            .context("failed to prompt user to select an option")?
        {
            _ = attach_to_session(path).context("failed to open session")?;
        }

        Ok(())
    }
}

fn parse_orbit_path() -> anyhow::Result<Vec<PathBuf>> {
    let orbit_path = env::var("ORBIT_PATH").unwrap_or_else(|_| String::new());
    let entries = orbit_path
        .split(':')
        .filter_map(|entry| {
            let path = Path::new(entry);
            path.is_dir().then_some(path)
        })
        .collect::<Vec<_>>();

    let mut projects = Vec::new();

    for path in entries {
        // The values in ORBIT_PATH were set explicitly by the user;
        // if invalid, return an error so the user knows they need to fix it.
        let directory = path.read_dir().context("failed to read directory")?;

        for result in directory {
            let entry = match result {
                Ok(entry) => entry,
                Err(err) => {
                    error!("failed to read entry in {}: {err}", path.display());
                    continue;
                }
            };

            let entry_path = entry.path();

            match fs::metadata(&entry_path) {
                Ok(metadata) if metadata.is_dir() => {
                    projects.push(entry_path);
                }
                Ok(_) => {} // Not a directory.
                Err(err) => {
                    error!(
                        "failed to determine if {} is a directory: {err}",
                        entry_path.display()
                    );
                }
            }
        }
    }

    Ok(projects)
}

fn prompt_to_select_option(
    directories: &[PathBuf],
    query: &str,
) -> anyhow::Result<Option<PathBuf>> {
    // Must be a forward slash for Session Preview to work.
    const SEPARATOR: &str = "/";

    let sessions_active = list_sessions()
        .inspect_err(|err| error!("failed to get active sessions: {err}"))
        .unwrap_or_default();

    let mut options = directories
        .iter()
        .filter_map(|d| d.to_str())
        .map(|path| {
            let is_active = sessions_active
                .iter()
                .find(|&session| {
                    let path = Path::new(path);
                    *session == SessionId::from_directory(path).unwrap()
                })
                .is_some();

            if is_active {
                format!("●{SEPARATOR}{path}")
            } else {
                format!("○{SEPARATOR}{path}")
            }
        })
        .collect::<Vec<_>>();

    options.sort();
    options.reverse();

    let echo = Command::new("echo")
        .arg(options.join("\n"))
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to execute echo in child process")?;

    let fzf = Command::new("fzf")
        .args([
            "--delimiter",
            SEPARATOR,
            "--with-nth",
            "{1}  {-2..}",
            "--preview",
            r#"
                session_name="$(orbit util session-id {2..})";
                tmux has-session -t "=$session_name" 2> /dev/null \
                    && tmux capture-pane -pt "$session_name" 2> /dev/null \
                    || echo "Session not running."
            "#,
            "--preview-window",
            "nohidden",
            "--preview-label",
            "Session Preview",
            "--border",
            "--tmux=center,90%,75%,border-native",
            "--query",
            query,
            "--select-1",
        ])
        .stdin(Stdio::from(echo.stdout.unwrap()))
        .output()
        .context("failed to execute fzf in child process")?;

    let selected = String::from_utf8(fzf.stdout).context("selected option is not valid UTF-8")?;

    match selected.split_once(SEPARATOR) {
        Some((_, path)) => {
            let path = path.strip_suffix('\n').unwrap();
            Ok(Some(PathBuf::from(path)))
        }
        None => Ok(None),
    }
}
