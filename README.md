# jvc: java versions controller

![Amount of downloads](https://img.shields.io/github/downloads/neculai-stanciu/jvc/total.svg?style=flat) ![Issues opened](https://img.shields.io/github/issues/neculai-stanciu/jvc) ![Github stars](https://img.shields.io/github/stars/neculai-stanciu/jvc) ![Build status](https://img.shields.io/github/workflow/status/neculai-stanciu/jvc/release-jvc)

A tool to handle java versions.
> **NOTE** This is a pre-release version - changes may appear in commands and options

Main objective with this tool is to offer a cross platform java versions manager tool. Inspired by `fnm` and `rustup` I tried to create something useful for my day to day development.

## Features

- Cross platform (Windows, Linux, MacOS)
- Support for AdoptOpenJDK and Zulu
- Install jdk in less than a minute

## Usage

![Usage usecase](./docs/usage.gif)

## Installation

### Using cargo

> For linux it is necessary to install **openssl-sys** first.
> To install on Ubuntu:
>
> - **`sudo apt update && sudo apt install libssl-dev`**
> - **`sudo apt update && sudo apt install pkg-config`**

Install jvc in Windows, Linux or MacOS using cargo install:

```bash
cargo install jvc --bin=jvc
```

### With installation script

#### For linux/macos

  ```bash
  curl -fsSL https://raw.githubusercontent.com/neculai-stanciu/jvc/main/.ci/install.sh | bash
  ```

#### For windows

- Download `jvc-init` binary:

  ```powershell
  Invoke-WebRequest -Uri "https://github.com/neculai-stanciu/jvc/releases/latest/download/jvc-init.exe" -OutFile "jvc-init.exe"
  ```

- Execute init:

  ```powershell
  .\jvc-init.exe
  ```

- Prepare jvc:

  ```bash
  jvc setup
  ```

- Close all terminal/powershell windows

## Supported shells

For now we support the following shells:

- bash
- zsh
- PowerShell
- CMD
