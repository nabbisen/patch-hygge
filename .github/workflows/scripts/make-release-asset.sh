#!/usr/bin/env bash
# Builds the release and creates an archive and optionally deploys to GitHub.
set -ex

if [[ -z "$GITHUB_REF" ]]
then
  echo "GITHUB_REF must be set"
  exit 1
fi
# Strip (repo)-refs/tags/ from the start of the ref.
TAG=${GITHUB_REF#*/tags/}

host=$(rustc -Vv | grep ^host: | sed -e "s/host: //g")
target=$2
if [ "$host" != "$target" ]
then
  export "CARGO_TARGET_$(echo $target | tr a-z- A-Z_)_LINKER"=rust-lld
fi
export CARGO_PROFILE_RELEASE_LTO=true
bin_name=$4
env NO_STRIP=1 npm run tauri build --locked -- --target $target

artifact=$bin_name@$TAG
workdir="$(pwd)"
build_dist=src-tauri/target/$target/release

cd $build_dist
mkdir $artifact

os_tag=$3
case $1 in
  ubuntu*)
    cp $bin_name $artifact/
    asset="$bin_name-$os_tag-$TAG.tar.gz"
    tar czf $workdir/$asset $artifact
    ;;
  macos*)
    cp $bin_name $artifact/
    asset="$bin_name-$os_tag-$TAG.tar.gz"
    # There is a bug with BSD tar on macOS where the first 8MB of the file are
    # sometimes all NUL bytes. See https://github.com/actions/cache/issues/403
    # and https://github.com/rust-lang/cargo/issues/8603 for some more
    # information. An alternative solution here is to install GNU tar, but
    # flushing the disk cache seems to work, too.
    sudo /usr/sbin/purge
    tar czf $workdir/$asset $artifact
    ;;
  windows*)
    cp $bin_name.exe $artifact/
    asset="$bin_name-$os_tag-$TAG.zip"
    7z a -w $workdir/$asset $artifact
    ;;
  *)
    echo "OS should be first parameter, was: $1"
    ;;
esac

cd $workdir

if [[ -z "$GITHUB_ENV" ]]
then
  echo "GITHUB_ENV not set, run: gh release upload $TAG target/$asset"
else
  echo "APP_TAG=$TAG" >> $GITHUB_ENV
  echo "APP_ASSET=$asset" >> $GITHUB_ENV
fi