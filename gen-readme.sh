#!/bin/sh

set -xe

cargo rdme -w bauer

# fix the attribute links
sed -i 's|Builder#|https://docs.rs/bauer/latest/bauer/derive.Builder.html#|' README.md
