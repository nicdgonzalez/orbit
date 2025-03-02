use std::{env, fs, path, process};

const TEMPLATE: &'static str = include_str!("../template.txt");

#[derive(clap::Args)]
pub struct Args {
    /// Start the finder with the given query.
    #[arg(default_value_t = String::from(""))]
    pub query: String,
}

pub fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let directories = parse_orbit_path()?;
    let options = directories.join("\n");

    let option = match prompt_to_select_option(&options, &args.query) {
        Some(directory) => path::PathBuf::from(directory),
        None => return Ok(()),
    };

    // Sanitize the directory name to ensure it can be used as a session name.
    let session_name = &option
        .file_name()
        .unwrap() // Directory names can not end in `..` by this point.
        .to_str()
        .expect("expected directory name to valid unicode")
        .chars()
        .map(|c| if !c.is_alphanumeric() { '-' } else { c })
        .collect::<String>();

    // Create a new session if one does not already exists.
    let tmux = process::Command::new("tmux")
        .args(["has-session", "-t", &format!("={session_name}")])
        .stderr(process::Stdio::null())
        .status()
        .expect("failed to execute tmux command");

    if let Some(1) = tmux.code() {
        create_tmux_session(&session_name, &option)?;
    }

    // Attach to the target session.
    let command = if env::var("TMUX").is_ok() {
        "switch-client"
    } else {
        "attach-session"
    };

    process::Command::new("tmux")
        .args([command, "-t", &format!("={session_name}")])
        .spawn()
        .expect("failed to execute tmux command")
        .wait()
        .expect("expected command to be running");

    Ok(())
}

fn parse_orbit_path() -> Result<Vec<String>, AttachError> {
    let orbit_path = env::var("ORBIT_PATH").unwrap_or_else(|_| "".to_string());
    let paths = orbit_path.split(":").collect::<Vec<_>>();

    let mut directories = Vec::<String>::new();

    for path in paths {
        let path = std::path::Path::new(path);

        if !path.is_dir() {
            return Err(AttachError::InvalidPath(path.display().to_string()));
        }

        let entries = path
            .read_dir()?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_ok_and(|ft| ft.is_dir()))
            .map(|d| d.path().display().to_string())
            .collect::<Vec<_>>();

        directories.extend(entries);
    }

    Ok(directories)
}

fn prompt_to_select_option(options: &str, query: &str) -> Option<String> {
    let echo = process::Command::new("echo")
        .arg(options)
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("failed to execute echo command");

    let fzf = process::Command::new("fzf")
        .args(["--query", query, "--select-1"])
        .stdin(process::Stdio::from(echo.stdout.unwrap()))
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("failed to execute fzf command");

    let output = fzf.wait_with_output().expect("failed to wait for output");
    let option = String::from_utf8(output.stdout).unwrap();

    (!option.is_empty()).then(|| option.trim_end().to_string())
}

fn create_tmux_session<P>(name: &str, directory: P) -> Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<path::Path>,
{
    // Create the session.
    let tmux = process::Command::new("tmux")
        .args([
            "new-session",
            "-d",
            "-s",
            &name,
            "-c",
            directory
                .as_ref()
                .to_str()
                .expect("expected path to be valid unicode"),
        ])
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()
        .expect("failed to execute tmux command");

    if !tmux.success() {
        panic!("failed to create tmux session");
    }

    // If there is a custom orbit script in the target directory, use it.
    let mut script = directory.as_ref().join("orbit.sh");

    // Otherwise, check for the fallback in the user's config directory.
    if !script.exists() {
        script = dirs::config_dir()
            .expect("unable to determine the user's config directory")
            .join("orbit")
            .join("orbit.sh");

        // If there is no fallback, create it.
        if !script.exists() {
            fs::create_dir_all(&script.parent().unwrap())?;
            fs::write(&script, TEMPLATE)?;
        }
    }

    // Run the script from inside the target tmux session.
    let status = process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &name,
            &format!("cat '{}' | bash", script.display()),
            "Enter",
        ])
        .status()
        .expect("failed to execute tmux command");

    if !status.success() {
        panic!("failed to run configuration script");
    }

    Ok(())
}

/// There was a problem attaching the client to a tmux session.
#[derive(Debug)]
pub enum AttachError {
    InvalidPath(String),
    ReadDirError(std::io::Error),
}

impl std::error::Error for AttachError {}

impl std::fmt::Display for AttachError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath(p) => write!(f, "expected path to be a directory: {p}"),
            Self::ReadDirError(err) => write!(f, "failed to read directory contents: {}", err),
        }
    }
}

impl From<std::io::Error> for AttachError {
    fn from(value: std::io::Error) -> Self {
        AttachError::ReadDirError(value)
    }
}
