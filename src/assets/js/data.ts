import {get, writable} from "svelte/store";
import Dexie, {type Table} from 'dexie';
import {fetchWithTimeout} from "./fetch";
import {cloudSettings} from "./settings";

interface DayEntry {
    import_tags: string[],
    // Duration in minutes
    duration: number,
    tags: string[],
    project: string,
    title: string,
    description: string
}

interface OneDay {
    entries: DayEntry[],
    // Show a warning when fewer hours are entered
    expected_min_hours: number,
    sick: boolean,
    holiday: boolean
}

interface OneMonth {
    days: OneDay[],
    year: number,
    month: number,
    created: number,
    change_id: number
}

interface LoadSheetResult {
    cloud: boolean,
    month: OneMonth
}

export type {DayEntry, OneDay, OneMonth}

export class TimeSheetDB extends Dexie {
    // 'months' is added by dexie when declaring the stores()
    // We just tell the typing system this is the case
    months!: Table<OneMonth>;

    constructor() {
        super('tsDB');
        this.version(1).stores({
            months: '[year+month]' // Primary key and indexed props
        });
    }
}

export const db = new TimeSheetDB();

/// Try to load sheet from cloud. If that fails, load it from local storage. If that fails, create a new entry.
export async function loadSheet(year: number, month: number): Promise<LoadSheetResult> {
    // cloud
    const s = get(cloudSettings);
    if (s.cloud_url && s.cloud_api_key) {
        try {
            const response = await fetchWithTimeout(s.cloud_url + `/api/timesheets/${year}_${month}`, {});
            const dbEntry: OneMonth = await response.json();
            if (dbEntry.created !== undefined && dbEntry.change_id !== undefined)
                return {cloud: true, month: dbEntry};
        } catch (e) {
            console.log("Failed to fetch sheet from cloud", year, month, e);
        }
    }

    // local
    let dbEntry = await db.months.where({year, month}).first();
    if (dbEntry) {
        if (dbEntry.created === undefined)
            dbEntry.created = Date.now();
        if (dbEntry.change_id === undefined)
            dbEntry.change_id = 1;
        else
            dbEntry.change_id += 1;
        return {cloud: false, month: dbEntry};
    }

    const daysInMonth = new Date(year, month + 1, 0).getDate();
    const translateToMondayFirstDay = [6, 0, 1, 2, 3, 4, 5];
    const dayEntries = new Array<OneDay>;
    for (let i = 1; i <= daysInMonth; ++i) {
        const dayDate = new Date(year, month, i);
        const dayOfWeek = translateToMondayFirstDay[dayDate.getDay()];
        const expected_min_hours = dayOfWeek < 5 ? 8 : 0
        const dayEntry: OneDay = {holiday: false, sick: false, expected_min_hours, entries: []};
        dayEntries.push(dayEntry);
    }
    dbEntry = {days: dayEntries, year, month, created: Date.now(), change_id: 1};
    return {cloud: false, month: dbEntry};
}

export async function storeSheet(sheet: OneMonth): Promise<void> {
    console.log("store", sheet);
    if (sheet.created === undefined)
        sheet.created = Date.now();
    if (sheet.change_id === undefined)
        sheet.change_id = 1;
    await db.months.put(sheet);
}

interface SelectedDate {
    date: number
}

export type {SelectedDate};
export const selectedDate = writable<SelectedDate>({date: Date.now()});