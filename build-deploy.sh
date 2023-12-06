#!/bin/sh
npm run build
cp -r build/* homeassistant-package/build/
TARGET_CC=aarch64-linux-gnu-gcc cargo build --target aarch64-unknown-linux-musl
cp ./target/aarch64-unknown-linux-musl/debug/timesheet-backend homeassistant-package/usr/bin/timesheet-backend
llvm-strip homeassistant-package/usr/bin/timesheet-backend
scp -r homeassistant-package/* root@homeassistant.local:/root/addons/timesheet-web/