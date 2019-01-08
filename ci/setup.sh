set -ex

platform=$(uname)
case $platform in
    Darwin)
        brew install sqlite
        ;;
    Linux)
        sudo apt-get install -y libsqlite3-0 libsqlite3-dev
        ;;
esac