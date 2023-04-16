# Timesheet web tool

A personal project to quickly create monthly timesheets and render them for pdf printing.
Outlook calendar entries can be imported and Gitlab activity is shown for each day.

An optional API backend is used for cloud syncronization and Outlook calendar imports.
The http backend is written in Rust using Axum for the http server and `ical` as well as `rrule` crates for
parsing Internet Calendar Scheduling (ics) Outlook URLs.

The web frontend is based on Svelte and SvelteKit/Vite. The sources are written in Typescript,
styling is applied in Scss.