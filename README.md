# loc

[![Crates.io](https://img.shields.io/crates/v/laputa-loc.svg)](https://crates.io/crates/laputa-loc)
[![License](https://img.shields.io/crates/l/laputa-loc.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88+-blue)](#)

The fastest and most accurate code counter around. Think that's wrong? Open an issue with [numbers](#benchmarks) or [a miscounted file](#accuracy) and I'll change this line.

loc counts the code, comment, blank, and shebang lines in a codebase by language, and reports them as text or JSON. Use it for audits, comparisons, scripts, or to see how much code is in a directory.

## Features

- Counts files, lines, and bytes by language.
- Counts code embedded in other files by that code's own language rules, including:
  - Scripts and styles in HTML, Vue, Svelte, Astro, and HTML templates.
  - The HTML around `<?php ... ?>` blocks in PHP files.
  - Code blocks in Markdown, AsciiDoc, and reStructuredText.
- Respects `.gitignore` and `.ignore` files.
- Skips generated files, such as lockfiles, translations, IDE project files, and files marked `DO NOT EDIT`.
- Reads the linguist attributes in `.gitattributes` files, the same way GitHub does.
- Filters by language, glob pattern, and directory depth.
- Shows counts per file or per directory.
- Recognizes 103 languages.
- Handles UTF-8, UTF-16, and legacy text encodings.

## Install

### Download a binary

Pick the newest prerelease on the [releases page](https://github.com/joshuarli/loc/releases) and download the archive for your platform. Every release ships a `SHA256SUMS` file next to the archives; verify one after downloading:

```bash
sha256sum -c SHA256SUMS --ignore-missing
```

| Platform | Asset |
| --- | --- |
| Linux x86_64, static (musl) | `loc-x86_64-unknown-linux-musl-static.zip` |
| Linux ARM64, static (musl) | `loc-aarch64-unknown-linux-musl-static.zip` |
| macOS Apple Silicon | `loc-aarch64-apple-darwin.zip` |

The static Linux binaries run on any Linux, including Alpine. Windows is not supported.

On Linux and macOS, make the binary executable and move it to a directory on your `PATH`:

```bash
chmod +x loc-*
sudo mv loc-* /usr/local/bin/loc
```

### Install with Cargo

```bash
cargo install laputa-loc
```

This requires Rust 1.88 or later. Release binaries are built with the `dist` profile, so a binary downloaded from the releases page is the fastest option. The minimum Rust version only increases in minor releases, never in patch releases.

## Quick start

Count the current directory:

```bash
loc
```

Count several paths together:

```bash
loc src tests build.rs
```

Files are listed by the path you gave, such as `src/main.rs`. If you give the same path twice, or a path inside another one you gave, its files are counted once.

Count a list of files from standard input:

```bash
git ls-files -z | loc -
fd -e rs | loc -
```

The list can have one path per line or NUL-separated paths. Directories in the list are walked. The usual filters apply, except `.gitignore`, so hidden files are skipped unless you pass `-H`. A symbolic link to a file counts as that file. A symbolic link to a directory is followed only with `--symlinks`.

Show per-file counts for `src/` as JSON:

```bash
loc -v src -o json
```

Compare with an earlier release:

```bash
loc diff v1.0.0               # v1.0.0 against the working tree
loc diff v1.0.0 --to v2.0.0   # two revisions
```

For each language, `loc diff` shows the files, lines, code, comments, and blanks in both versions, and the change. Both sides count the files git tracks, with the same filters, so only real changes appear.

More examples:

```bash
loc -t 10                                         # only the 10 largest languages
loc -L rust                                       # only Rust
loc -e 'test_*' -e '*.md'                         # exclude glob patterns
loc --exclude-lang markdown --exclude-lang toml   # exclude languages
loc --symlinks -H                                 # follow symbolic links, include hidden files
loc --no-gitignore                                # include files .gitignore excludes
loc -D                                            # counts by directory
loc langs                                         # list supported languages
```

## Output formats

- `human` (default) for terminals.
- `json` or `json-compact` for scripts.

### JSON

Every level of a JSON report counts lines the same way. `paths` lists the paths that were counted. `files` appears with `--verbose`, and `directories` with `--by-dir`. `loc diff -o json` reports its `from` and `to` counts in the same shape.

A language or file that contains code in another language, such as Rust in a Markdown file, has an `embedded` list with each embedded language and its lines. Those lines are also included in the language's or file's own `lines`.

```json
"embedded": [{ "name": "Rust", "lines": { "total": 120, "code": 96, "comment": 14, "blank": 10, "shebang": 0 } }]
```

```json
{
	"schema_version": 2,
	"paths": ["src"],
	"totals": {
		"file_count": 30,
		"lines": { "total": 5956, "code": 5083, "comment": 331, "blank": 542, "shebang": 0 },
		"bytes": 195221
	},
	"languages": [
		{
			"name": "Rust",
			"file_count": 30,
			"lines": { "total": 5956, "code": 5083, "comment": 331, "blank": 542, "shebang": 0 },
			"bytes": 195221,
			"files": [{ "path": "cli.rs", "lines": { "total": 328, "code": 264, "comment": 39, "blank": 25, "shebang": 0 }, "bytes": 12873 }]
		}
	],
	"hidden_languages": 0
}
```

### Stability

Starting with 1.0, JSON fields and configuration file keys only change in a new major version.

`schema_version` increases whenever a JSON field is renamed, removed, or changes meaning. New fields can appear without a version change, so your scripts should ignore fields they don't recognize.

## Command-line options

Usage: `loc [OPTIONS] [PATH]...`

If you don't give a path, loc counts the current directory. If the path is `-`, it reads the paths to count from standard input.

- `-q, --quiet`: show only the totals.
- `-v, --verbose`: also show the counts for each file.
- `--no-gitignore`: ignore `.gitignore` files.
- `--no-gitattributes`: ignore the linguist attributes in `.gitattributes` files. See [Linguist attributes](#linguist-attributes).
- `--include-generated`: count generated files. See [Generated files](#generated-files).
- `--git`: count only the files that git tracks, including in submodules. `.gitignore` doesn't apply to tracked files.
- `--max-depth <N>`: descend at most N directory levels.
- `--min-lines <N>`: hide languages with fewer than N lines.
- `-H, --hidden`: include hidden files and directories. The `.git` directory is always skipped.
- `--symlinks`: follow symbolic links. A link to one of its own parent directories is skipped and reported as an error.
- `-e, --exclude <PATTERN>`: exclude files and directories that match a glob pattern. Repeatable.
- `-L, --lang <LANGUAGE>`: count only this language. Repeatable, or comma-separated. Can't be combined with `--exclude-lang`.
- `--exclude-lang <LANGUAGE>`: don't count this language. Repeatable, or comma-separated. Can't be combined with `--lang`.
- `--ext <EXT>`: count only files with this extension, such as `rs` or `py`. Repeatable, or comma-separated.
- `-n, --number-style <plain|comma|underscore|space>`: how to format numbers. Default: `plain`.
- `-u, --size-units <binary|decimal>`: the units for sizes. Default: `binary`.
- `-p, --precision <0-6>`: decimal places in percentages. Default: `1`.
- `-s, --sort-by <lines|code|comments|blanks|files|size|name>`: how to sort languages and files. Default: `lines`.
- `-d, --sort-direction <asc|desc>`: the sort direction. Default: `desc`.
- `-t, --top-languages <N>`: show only the N largest languages. N must be at least 1; to show only the totals, use `--quiet`.
- `-D, --by-dir`: show counts by directory instead of by language.
- `-o, --output <human|json|json-compact>`: the output format. Default: `human`.
- `--indent <tab|1-8>`: the indentation of human-readable output, as a tab or a number of spaces. Default: `tab`.
- `--fail-on-error`: exit with an error if any file couldn't be read.
- `-c, --config <PATH>`: use this configuration file.
- `--no-config`: don't read any configuration file.
- `-h, --help`: show help.
- `-V, --version`: show the version, including the commit the binary was built from.

### Exit status

- `0`: success.
- `1`: an error, including a file that couldn't be read when you pass `--fail-on-error`.
- `2`: invalid arguments.

## Subcommands

- `diff <FROM> [PATH]`: compare counts by language between a git revision and the working tree, or another revision with `--to <REV>`.
- `langs`: list the supported languages.
- `init`: write a configuration file that lists every setting.

## Configuration

loc can read settings from a TOML file. Options on the command line override the file. An unknown key in the file is an error, so typos don't go unnoticed.

loc uses the first of these files that exists:

1. The file you pass with `--config <path>`. It's an error if this file doesn't exist.
2. `./.loc.toml`
3. `./loc.toml`
4. On Linux, `${XDG_CONFIG_HOME:-~/.config}/loc/config.toml`; on macOS, `~/Library/Application Support/loc/config.toml`.
5. `~/.loc.toml`

To ignore all of these files, for example in a benchmark or a script that has to behave the same on every machine, pass `--no-config`.

### Example configuration

```toml
paths = ["src", "tests"]  # Relative to this file.

[analysis]
verbosity = "verbose"
respect_gitignore = true
respect_gitattributes = true  # Use the linguist attributes in .gitattributes files.
git_tracked_only = false  # true counts only the files git tracks.
include_hidden = true
follow_symlinks = false
include_generated = false
max_depth = 5
exclude_patterns = ["languages.json5", "*.tmp", "node_modules/*"]  # File names or glob patterns.
include_languages = ["rust", "python"]  # Count only these languages.
# exclude_languages = ["markdown", "toml"]  # Or skip these. You can't set both.
include_extensions = ["rs", "py"]  # Count only these extensions.
fail_on_error = false

[display]
number_style = "comma"
size_units = "decimal"
precision = 4
sort_by = "files"
sort_direction = "desc"
output = "human"
top_languages = 10
min_lines = 100
by_dir = false
indent = "tab"
```

## Generated files

By default, loc skips generated files. To count them, pass `--include-generated`.

A file counts as generated in any of these cases:

- **Its linguist attribute says so.** See [Linguist attributes](#linguist-attributes).
- **Its name says so.** These files count as generated by their names:
  - Lockfiles, such as `Cargo.lock` and `package-lock.json`.
  - Minified files, such as `*.min.js` and `*.min.css`.
  - Translations, which tools extract and merge: Gettext (`*.po`, `*.pot`), XLIFF (`*.xliff`, `*.xlf`), .NET resources (`*.resx`), and Group Policy language files (`*.adml`).
  - Files that IDEs and interface designers save: Visual Studio solutions (`*.sln`) and filters (`*.vcxproj.filters`), per-user project settings (`*.csproj.user`), Visual C++ 2008 projects (`*.vcproj`), ReSharper settings (`*.DotSettings`), IntelliJ modules (`*.iml`), Eclipse projects (`.project`, `.classpath`, `.cproject`), Interface Builder files (`*.xib`, `*.storyboard`), and Qt Designer and Glade files (`*.ui`, `*.glade`).
- **It's a project file Visual Studio wrote.** A `.vcxproj` file, or a `.csproj`, `.vbproj`, or `.fsproj` file from before SDK-style projects, has the `http://schemas.microsoft.com/developer/msbuild/2003` namespace. An SDK-style project, such as `<Project Sdk="Microsoft.NET.Sdk">`, is written by hand and counts as MSBuild.
- **Its header says so.** loc reads the comments at the start of each file, up to the first line of code, and looks for a note that a tool wrote the file. For example:
  - `// Code generated by stringer; DO NOT EDIT.` (Go)
  - `// Generated by the protocol buffer compiler.  DO NOT EDIT!` (protobuf)
  - `// <auto-generated />` (C#)
  - `// @generated by pegen`, or any other `@generated`
  - `/* automatically generated by rust-bindgen */`
  - `# This makefile was generated by bakefile.` followed closely by `# Do not modify`

A comment that only mentions generated code, such as `// Retrieves the auto-generated key`, doesn't make a file generated.

## Linguist attributes

GitHub uses the linguist attributes in `.gitattributes` files for its language statistics, and loc uses them the same way:

| Attribute | Effect |
| --- | --- |
| `linguist-generated` | The file is generated, so it's skipped unless you pass `--include-generated`. |
| `-linguist-generated` or `linguist-generated=false` | The file isn't generated, even if its name or header looks like it is. |
| `linguist-vendored` | The file is skipped. |
| `linguist-documentation` | The file is skipped. |
| `linguist-language=Name` | The file is counted as `Name`, with the exceptions below. |

loc ignores `linguist-language` when:

- It doesn't recognize `Name`.
- The file's name already identifies exactly one language. For example, `*.svelte` files stay Svelte.

When the file's name matches no language or several, such as `*.h`, `Name` decides.

For example:

```gitattributes
**/zz_generated.*.go linguist-generated
lib/libc/**          linguist-vendored
*.fixed              linguist-language=Rust
```

loc reads the `.gitattributes` files in the directories it counts, and in the directories above them up to the repository root. A file in a deeper directory overrides one higher up, and within a file, later lines override earlier ones. Macros, such as `[attr]generated linguist-generated`, are expanded.

With `--git` and `loc diff`, the working tree's `.gitattributes` files apply to every revision, so both sides of a comparison follow the same rules.

To ignore linguist attributes, pass `--no-gitattributes`.

## Technical notes

### Memory-mapped files

loc memory-maps files of 256 KiB or larger, which makes reading them faster. Don't modify files while loc is counting them.

### Threads

loc counts files on several threads, each keeping its own totals, and combines them at the end. You can run several loc processes on different directories at once, but don't run more than one on the same directory.

## Accuracy

Every change is checked in CI against [LineJudge](https://github.com/loc-conformance/linejudge), a collection of tricky files: comment markers in strings, raw strings, heredocs, languages embedded in other languages, and more. LineJudge checks loc against the counting rules declared in [`.linejudge`](.linejudge).

Each tool is judged against its own declared rules, so a failure means that the tool breaks its own rules. Results with LineJudge 0.3.0:

| Tool | Cases that agree, out of 95 |
| --- | ---: |
| loc | **95** |
| mezura 3.4.0 | 89 |
| scc 4.1.0 | 71 |
| tokei 15.0.0 | 45 |

loc's answers are recorded in [`.linejudge/recorded/loc.toml`](.linejudge/recorded/loc.toml), and CI fails if any of them changes.

To run LineJudge locally:

1. Install LineJudge.
2. Run `cargo build --release`.
3. Run `linejudge check --counter loc`.

## Benchmarks

All times are for files already in the file cache, on an AMD Ryzen 7 PRO 5850U (8 cores, 16 threads) with 32 GB of RAM. The loc numbers were measured with the x86-64 glibc binary built at commit `d3c937a`; current releases ship a musl static binary instead.

### Equal work

In these tables, every tool counts the same files. [linebench](https://github.com/loc-conformance/linebench) pins each repository's commit, turns off each tool's filters, and checks that each tool's file and line counts are within 1% of the repository's.

Median time:

| Repository | Files | Lines | loc | mezura | scc | tokei |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| [cpython](https://github.com/python/cpython) | 3,554 | 2.2M | **15 ms** | 21 ms | 51 ms | 106 ms |
| [jdk](https://github.com/openjdk/jdk) | 61,420 | 13M | **109 ms** | 125 ms | 305 ms | 767 ms |
| [linux](https://github.com/torvalds/linux) | 63,738 | 36M | **169 ms** | 188 ms | 574 ms | 995 ms |

Median peak memory:

| Repository | loc | mezura | scc | tokei |
| --- | ---: | ---: | ---: | ---: |
| [cpython](https://github.com/python/cpython) | **13 MB** | 16 MB | 47 MB | 18 MB |
| [jdk](https://github.com/openjdk/jdk) | **19 MB** | 26 MB | 94 MB | 56 MB |
| [linux](https://github.com/torvalds/linux) | **87 MB** | 140 MB | 347 MB | 251 MB |

To reproduce these tables, run the following command with linebench 0.4.1, after running `linebench fetch --counters all --corpus all`:

```sh
linebench run cpython,linux,jdk --counters scc,tokei,mezura,loc@dev --given loc@dev=path/to/loc --definition loc@dev=.linebench/loc.toml --add .linebench
```

Each tool ran 30 times, half of them in reverse order. These tables measure loc, mezura 3.5.0, scc 4.1.0, and tokei 15.0.0, the versions linebench 0.4.1 fetches.

### Default settings

In these tables, every tool runs with no flags, which is how most people use it. The tools don't count the same files, so compare these times with care. For example, mezura counts 40,234 files in rust, where loc counts 44,338. The file and line counts in the table are from loc.

Median time:

| Repository | Files | Lines | loc | mezura | tokei | scc | gocloc |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| [ripgrep](https://github.com/BurntSushi/ripgrep) | 156 | 65K | **3.7 ms** | 6.9 ms | 10 ms | 13 ms | 41 ms |
| [cli](https://github.com/cli/cli) | 1,102 | 312K | **7.1 ms** | 9.1 ms | 23 ms | 20 ms | 113 ms |
| [nvda](https://github.com/nvaccess/nvda) | 1,088 | 298K | **7.4 ms** | 9.1 ms | 49 ms | 22 ms | 136 ms |
| [wxWidgets](https://github.com/wxWidgets/wxWidgets) | 4,141 | 1.7M | **24 ms** | 25 ms | 147 ms | 105 ms | 448 ms |
| [cpython](https://github.com/python/cpython) | 4,828 | 2.5M | **22 ms** | 26 ms | 139 ms | 81 ms | 663 ms |
| [kubernetes](https://github.com/kubernetes/kubernetes) | 22,681 | 5.2M | **59 ms** | 61 ms | 355 ms | 217 ms | 2.00 s |
| [rust](https://github.com/rust-lang/rust) | 44,338 | 5.1M | **90 ms** | 110 ms | 460 ms | 227 ms | 2.41 s |
| [linux](https://github.com/torvalds/linux) | 92,882 | 44M | **239 ms** | 257 ms | 1.45 s | 952 ms | 11.0 s |

Median peak memory:

| Repository | loc | mezura | tokei | scc | gocloc |
| --- | ---: | ---: | ---: | ---: | ---: |
| [ripgrep](https://github.com/BurntSushi/ripgrep) | **7.0 MB** | 7.8 MB | 7.6 MB | 27 MB | 29 MB |
| [cli](https://github.com/cli/cli) | 7.9 MB | 8.7 MB | **7.9 MB** | 32 MB | 36 MB |
| [nvda](https://github.com/nvaccess/nvda) | 9.2 MB | **9.2 MB** | 33 MB | 39 MB | 38 MB |
| [wxWidgets](https://github.com/wxWidgets/wxWidgets) | 19 MB | **19 MB** | 127 MB | 75 MB | 40 MB |
| [cpython](https://github.com/python/cpython) | **16 MB** | 20 MB | 23 MB | 66 MB | 46 MB |
| [kubernetes](https://github.com/kubernetes/kubernetes) | **18 MB** | 21 MB | 67 MB | 98 MB | 53 MB |
| [rust](https://github.com/rust-lang/rust) | **30 MB** | 45 MB | 90 MB | 129 MB | 67 MB |
| [linux](https://github.com/torvalds/linux) | **91 MB** | 189 MB | 268 MB | 379 MB | 110 MB |

Each tool ran 20 times, or 10 times for rust and linux, in rounds of two runs per tool. The tools' order flips each round, with a pause between rounds, so a processor that slows down as it heats up doesn't favor one tool.

To reproduce these tables, run `LOC=path/to/loc scripts/bench.sh`, which pins each repository's commit. It uses hyperfine 2.0.0 and these tools: mezura 3.5.0, tokei 15.0.0, scc 4.1.0, and gocloc 0.7.0.

## License

loc is licensed under the [MIT License](LICENSE).
