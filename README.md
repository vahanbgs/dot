# dot
A simple dotfile manager written in Rust

`dot` helps you keep your configuration files under version control, synchronize them across machines, and deploy them into your home directory.
It also supports template files with machine-specific variables, and Nuke documents, which let one
value be written once and rendered into every format that needs it.

## Features

* Store dotfiles in a Git repository
* Track files directly from your home directory
* Deploy tracked files back into `$HOME`
* No symlinks, no unintended modifications to your dot files
* Template support for machine-specific configuration
* Nuke documents rendered to JSON, YAML, TOML, XML, KDL, Lua, INI, gitconfig, Nix, Ghostty and plist
* Git workflow shortcuts (`add`, `commit`, `push`, `pull`, `status`)
* Open tracked files in your preferred editor
* Optional automatic deployment after edits and pulls
* Shell completion generation

## Installation

### From Source

```sh
cargo install --git https://github.com/vahanbgs/dot.git
```

## Nix Flake

This repository can be used directly as a Nix flake, allowing you to build, run, or install dot using Nix.

Execute the latest version directly from the repository:

```sh
nix run github:vahanbgs/dot
```

Build the package locally:

```sh
nix build github:vahanbgs/dot
```

The resulting executable will be available under `result/bin/dot`.

Add the repository as a flake input:

```sh
{
  inputs = {
    dot.url = "github:vahanbgs/dot/main";
    dot.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, dot }:
    {
      # ...
    };
}
```

## Repository Layout

After initialization, the managed repository is typically stored in:

```text
~/.local/share/dot/
```

Tracked files are kept under:

```text
home/
```

Example:

```text
home/
├── .bashrc
├── .gitconfig
└── .config/
    ├── palette.nuke
    ├── nvim/
    │   └── init.lua
    └── alacritty/
        └── alacritty.toml.nuke
```

When deployed, these files are copied into the corresponding locations under your home directory.

## Quick Start

### Initialize a Dotfiles Repository

If you already have a dotfiles repository with the expected structure, you can just clone and deploy it in one command.

```bash
dot init --deploy https://github.com/vahanbgs/dotfiles.git
```

Otherwise, create a Git repository in the appropriate location:

```sh
git init "$XDG_DATA_HOME"/dot/
```

### Track a File

```bash
dot track ~/.bashrc
```

This copies the file into the managed repository while preserving its path relative to your home directory.

### Deploy Files

```bash
dot deploy
```

Copies all tracked files from the repository into your home directory.

### Edit Files

```sh
dot edit ~/.bashrc
```

Edits the corresponding file in your repository instead of your home directory using your default editor.

### Repository Access

```sh
dot cd
```

Launches a subshell in the directory of your repository.

## Nuke Documents

A file ending in `.nuke` is a [Nuke](https://github.com/vahanbgs/nuke) document. It is evaluated and
rendered to its target's format on deployment, and the `.nuke` is dropped from the name:

```text
home/.config/alacritty/alacritty.toml.nuke   ->  ~/.config/alacritty/alacritty.toml
```

The extension underneath the `.nuke` names the format, so `alacritty.toml.nuke` is TOML. Eleven are
available: `json`, `yaml`, `toml`, `xml`, `kdl`, `lua`, `ini`, `gitconfig`, `nix`, `ghostty` and
`plist`.

The point is that documents can import each other, so a value is written once:

```text
# home/.config/palette.nuke
{
	accent = "#FE8019"
	background = "#282828"
}
```

```text
# home/.config/alacritty/alacritty.toml.nuke
palette := @import "../palette.nuke"

{
	colors = {primary = {background = palette.background cursor = palette.accent}}
}
```

Editing `palette.nuke` and running `dot deploy` changes every file that reads it.

### Modules

A `.nuke` file whose name does not give a format is a **module**: something other documents import,
never something deployed. `palette.nuke` above is one, so it feeds the tree without appearing in
your home directory.

### Files Named Rather Than Extended

`.gitconfig` and Ghostty's `config` carry no extension to read a format from, so name their target
in the **manifest**, `dot.nuke` at the root of the repository beside `home/`, keyed by the deployed
path relative to `$HOME`:

```text
# ~/.local/share/dot/dot.nuke
{
	targets = {
		".config/git/config"     => Gitconfig
		".config/ghostty/config" => Ghostty
	}
}
```

This table is consulted before the extension, so it also overrides a name that would say otherwise.

The manifest is itself a Nuke document, read through `bind::from_path`, so a target that does not
exist is refused when `dot` loads it — naming the line that misspelled it — rather than surfacing
later against whichever file wanted it. It lives beside `home/` rather than inside it, so it is
versioned with the tree it describes and is never deployed. It is not `config.toml`: that file says
how this machine behaves, and the manifest says what this tree contains.

### Templates or Documents

The two live side by side and do different jobs. Nuke writes what has a grammar; templates write
what does not — a `config.fish`, a `flake.nix`, a shell profile. Neither replaces the other.

## Templates

Files can be stored as templates to support machine-specific values.

Track a file as a template:

```bash
dot track ~/.gitconfig --template
```

Template files use the `.tielpmet` extension inside the repository.

Example template:

```text
[user]
    name = (<|[ name ]|>)
    email = (<|[ email ]|>)
```

During deployment, `dot` will prompt for any missing variables:

```text
name: John Doe
email: john@example.com
```

Values are stored locally and reused for future deployments.

### Local Variables

Template variables are stored in:

```text
~/.config/dot/local.toml
```

Example:

```toml
name = "John Doe"
email = "john@example.com"
```

This file should typically not be committed to version control.

## Automatic Deployment

Create:

```text
~/.config/dot/config.toml
```

```toml
auto_deploy = true
```

When enabled, commands such as `edit` and `pull` automatically trigger deployment unless explicitly disabled with `--no-deploy`.

### Git Shortcuts

```bash
dot add [args...]
dot commit [args...]
dot push [args...]
dot pull
dot status
```

These commands execute the corresponding Git command inside the managed repository.

### Generate Shell Completions

```bash
dot completions bash
dot completions zsh
dot completions fish
```

## Configuration

Two files, with different scopes. How this machine behaves:

```text
~/.config/dot/config.toml
```

```toml
auto_deploy = true
```

What the tracked tree contains, versioned with it — see **Files Named Rather Than Extended**:

```text
~/.local/share/dot/dot.nuke
```

## How It Works

1. Track files from your home directory.
2. Store them in a Git repository.
3. Commit and push changes as usual.
4. Pull the repository on another machine.
5. Deploy files into the local home directory.
6. Fill in machine-specific values using templates.
7. Share values across formats using Nuke documents.
