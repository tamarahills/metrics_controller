# This script must be sourced. Please run `source tools/mac-os-x-setup.sh`

# openssl libraries are already installed on OS X base system. That's
# why we need to explicitely specify the directories that brew installed.
export DEP_OPENSSL_INCLUDE="$(brew --prefix openssl)/include"
export OPENSSL_LIB_DIR="$(brew --prefix openssl)/lib"
