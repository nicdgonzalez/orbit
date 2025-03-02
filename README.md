# Orbit

> [!WARNING]
> The Rust implementation of this project was experimental and has less
> features than the main bash implementation. I'm leaving the branch in
> the repository for historical reasons.

**Orbit** is a personalized version of [tmux-sessionizer], allowing you to
quickly spin up pre-configured tmux sessions for your projects. It supports
running custom scripts and falls back to a global default when needed.

## Introduction to tmux

If you're new to tmux, here's a brief introduction:

- tmux (Terminal Multiplexer) is a software application that allows you to
  manage multiple terminal sessions from a single window.
- It enables you to create multiple windows, panes, and sessions, making it
  easier to work on multiple projects simultaneously.
- tmux is highly customizable, allowing you to tailor your workflow to your
  specific needs.

## Getting Started

To use Orbit, you'll need to install the following dependencies:

- [bash] (version 5.2 or higher) for executing configuration scripts,
- [tmux] (version 3.5a or higher),
- [fzf] (version 0.59 or higher) for fuzzy finding project directories,
- [cargo] (version 1.82 or higher) to build and install the project.

### Installation

Once you have the dependencies installed, you can install Orbit using cargo:

```bash
cargo install --git https://github.com/nicdgonzalez/orbit orbit
```

Verify the installation by running:

```bash
orbit --version
```

### Initial setup

Try running `orbit attach` to see the project selection prompt. Initially, the
list will be empty.

## Configuring Orbit

To populate the project list, you need to define the `ORBIT_PATH` environment
variable. This variable should contain a colon-separated list of directories,
similar to the `PATH` variable.

For example, if your project directory structure looks like this:

```
$HOME
└── projects
    ├── personal
    │   └── ...
    └── work
        ├── company_a
        │   ├── ...
        ├── company_b
        │   ├── ...
        └── company_c
            └── ...
```

You can add the following code to your `.bashrc` file to set the `ORBIT_PATH`
variable:

```bash
get_orbit_paths() {
    local personal="$HOME/projects/personal"

    # Each client has their own subdirectory with projects inside.
    local work="$(find "$HOME/projects/work" -mindepth 1 -maxdepth 1 -type d -printf "%p:" | sed 's/:$//')"

    echo "$personal:$work"
}
export ORBIT_PATH="$(get_orbit_paths)"
unset -f get_orbit_paths
```

With this configuration, running `orbit attach` will display a list of your
projects.

## Using the `attach` command

The attach command accepts an optional third argument, which allows you to pass
a query to the interactive finder. If the query matches only one option, Orbit
will automatically connect you to that session.

For a full list of available commands, try `orbit help`.

### Creating custom configuration scripts

When Orbit creates a new tmux session, it looks for an `orbit.sh` script at the
following locations:

1. The target project's root directory.
1. In the user's config directory (i.e., `$HOME/.config/orbit/orbit.sh`)
1. If there is no script in the config directory, one is created for you.

### Example use cases

- Use Orbit to manage multiple projects simultaneously, each with its own
  custom tmux session.
- Create a custom script to automate tasks for a specific project, and use
  Orbit to attach to the project's tmux session and run the script.
- Use Orbit with other tools, such as git or vim, to create a customized
  workflow.

## Troubleshooting

If you encounter issues with Orbit, here are some common problems and
solutions:

- **Orbit doesn't recognize my projects**: Make sure that the `ORBIT_PATH`
  variable is set correctly and that the project directories are in the correct
  location. Use the following command to print a list of directories Orbit
  should see:
  ```bash
  find $(echo "$ORBIT_PATH" | tr ':' ' ') -mindepth 1 -maxdepth 1 -type d
  ```

To report a bug or request a new feature, create a new issue
[here](https://github.com/nicdgonzalez/orbit/issues).

## Additional Resources

- [Repository (GitHub)](https://github.com/nicdgonzalez/orbit)
- [tmux Documentation](https://github.com/tmux/tmux/wiki)
- [Guide to learning tmux](https://hamvocke.com/blog/a-quick-and-easy-guide-to-tmux/)

[bash]: https://www.gnu.org/software/bash/
[cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[fzf]: https://github.com/junegunn/fzf
[tmux]: https://github.com/tmux/tmux
[tmux-sessionizer]: https://github.com/ThePrimeagen/tmux-sessionizer
