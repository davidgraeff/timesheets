# Timesheet web tool

A personal project to quickly create monthly timesheets and render them for pdf printing.
Outlook calendar entries can be imported and Gitlab activity is shown for each day.

An optional API backend is used for cloud syncronization and Outlook calendar imports.
The http backend is written in Rust using Axum for the http server and `ical` as well as `rrule` crates for
parsing Internet Calendar Scheduling (ics) Outlook URLs.

The web frontend is based on Svelte and SvelteKit/Vite. The sources are written in Typescript,
styling is applied in Scss.

# Cross compile to RPI4 on Fedora and install to home assistant

```
TARGET_CC=aarch64-linux-gnu-gcc cargo build --target aarch64-unknown-linux-musl
cp ./target/aarch64-unknown-linux-musl/debug/timesheet-backend homeassistant-package/usr/bin/timesheet-backend
llvm-strip homeassistant-package/usr/bin/timesheet-backend
scp -r homeassistant-package/* root@homeassistant.local:/root/addons/timesheet-web/
```