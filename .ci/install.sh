#!/bin/bash

INSTALL_DIR="$HOME/.jvc"
RELEASE="latest"
OS="$(uname -s)"

# Parse Flags
parse_args() {
  while [[ $# -gt 0 ]]; do
    key="$1"

    case $key in
    -d | --install-dir)
      INSTALL_DIR="$2"
      shift # past argument
      shift # past value
      ;;
    -s | --skip-shell)
      SKIP_SHELL="true"
      shift # past argument
      ;;
    -r | --release)
      RELEASE="$2"
      shift # past release argument
      shift # past release value
      ;;
    *)
      echo "Unrecognized argument $key"
      exit 1
      ;;
    esac
  done
}

set_filename() {
  if [ "$OS" == "Linux" ]; then
    FILENAME="jvc-linux-amd64"
  elif [ "$OS" == "Darwin" ]; then
    FILENAME="jvc-macos-amd64"
    echo "Downloading the latest jvc binary from GitHub..."
  else
    echo "OS $OS is not supported."
    exit 1
  fi
}

download_jvc() {
  if [ "$RELEASE" == "latest" ]; then
    URL="https://github.com/neculai-stanciu/jvc/releases/latest/download/$FILENAME"
  else
    URL="https://github.com/neculai-stanciu/jvc/releases/download/$RELEASE/$FILENAME"
  fi

  DOWNLOAD_DIR=$(mktemp -d)

  echo "Downloading $URL..."

  mkdir -p "$INSTALL_DIR" &>/dev/null

  if ! curl --progress-bar --fail -L "$URL" -o "$DOWNLOAD_DIR/$FILENAME"; then
    echo "Download failed.  Check that the release/filename are correct."
    exit 1
  fi

  if [ -f "$DOWNLOAD_DIR/$FILENAME" ]; then
    mv "$DOWNLOAD_DIR/$FILENAME" "$INSTALL_DIR/jvc"
  else
    echo "Cannot find $FILENAME in path $DOWNLOAD_DIR"
    exit 2
  fi

  chmod u+x "$INSTALL_DIR/jvc"
}

check_dependencies() {
  echo "Checking dependencies for the installation script..."

  echo -n "Checking availability of curl... "
  if hash curl 2>/dev/null; then
    echo "OK!"
  else
    echo "Missing!"
    SHOULD_EXIT="true"
  fi

  echo -n "Checking availability of unzip... "
  if hash unzip 2>/dev/null; then
    echo "OK!"
  else
    echo "Missing!"
    SHOULD_EXIT="true"
  fi

  if [ "$USE_HOMEBREW" = "true" ]; then
    echo -n "Checking availability of Homebrew (brew)... "
    if hash brew 2>/dev/null; then
      echo "OK!"
    else
      echo "Missing!"
      SHOULD_EXIT="true"
    fi
  fi

  if [ "$SHOULD_EXIT" = "true" ]; then
    exit 1
  fi
}

ensure_containing_dir_exists() {
  local CONTAINING_DIR
  CONTAINING_DIR="$(dirname "$1")"
  if [ ! -d "$CONTAINING_DIR" ]; then
    echo " >> Creating directory $CONTAINING_DIR"
    mkdir -p "$CONTAINING_DIR"
  fi
}

setup_shell() {
  CURRENT_SHELL="$(basename "$SHELL")"

  if [ "$CURRENT_SHELL" == "zsh" ]; then
    CONF_FILE=${ZDOTDIR:-$HOME}/.zshrc
    ensure_containing_dir_exists "$CONF_FILE"
    echo "Installing for Zsh. Appending the following to $CONF_FILE:"
    echo ""
    echo '  # jvc'
    echo '  export PATH='"$INSTALL_DIR"':$PATH'
    echo '  eval "`jvc env`"'

    echo '' >>$CONF_FILE
    echo '# jvc' >>$CONF_FILE
    echo 'export PATH='$INSTALL_DIR':$PATH' >>$CONF_FILE
    echo 'eval "`jvc env`"' >>$CONF_FILE

  elif [ "$CURRENT_SHELL" == "bash" ]; then
    if [ "$OS" == "Darwin" ]; then
      CONF_FILE=$HOME/.profile
    else
      CONF_FILE=$HOME/.bashrc
    fi
    ensure_containing_dir_exists "$CONF_FILE"
    echo "Installing for Bash. Appending the following to $CONF_FILE:"
    echo ""
    echo '  # jvc'
    echo '  export PATH='"$INSTALL_DIR"':$PATH'
    echo '  eval "`jvc env`"'

    echo '' >>$CONF_FILE
    echo '# jvc' >>$CONF_FILE
    echo 'export PATH='"$INSTALL_DIR"':$PATH' >>$CONF_FILE
    echo 'eval "`jvc env`"' >>$CONF_FILE

  else
    echo "Could not infer shell type. Please set up manually."
    echo "Current shell is $CURRENT_SHELL"
    exit 1
  fi

  echo ""
  echo "In order to apply the changes, open a new terminal or run the following command:"
  echo ""
  echo "  source $CONF_FILE"
}

parse_args "$@"
set_filename
check_dependencies
download_jvc
if [ "$SKIP_SHELL" != "true" ]; then
  setup_shell
fi
