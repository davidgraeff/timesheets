import {get, writable} from "svelte/store";
import Dexie, {type Table} from 'dexie';
import {fetchWithTimeout, post} from "./fetch";
import {cloudSettings, localSettings, cloudUrl} from "./settings";

interface DayEntry {
    import_tags: string[],
    // Duration in minutes
    duration: number,
    tags: string[],
    project: string[],
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

function fixMonth(dbEntry: OneMonth): OneMonth {
    for (const day in dbEntry.days) {
        for (const activityIndex in dbEntry.days[day].entries) {
            const activity = dbEntry.days[day].entries[activityIndex];
            if (!Array.isArray(activity.project)) dbEntry.days[day].entries[activityIndex].project = [];
            if (!activity.tags) dbEntry.days[day].entries[activityIndex].tags = [];
        }
    }
    return dbEntry;
}

/// Try to load sheet from cloud. If that fails, load it from local storage. If that fails, create a new entry.
export async function loadSheet(year: number, month: number): Promise<LoadSheetResult> {
    const monthStartWith1 = month + 1

    // cloud
    const s = get(cloudSettings);
    if (cloudUrl && s.cloud_api_key) {
        try {
            const response = await fetchWithTimeout(cloudUrl + `/timesheets/${year}_${monthStartWith1}`, {});
            const dbEntry: OneMonth = await response.json();
            if (dbEntry.created !== undefined && dbEntry.change_id !== undefined)
                return {cloud: true, month: fixMonth(dbEntry)};
        } catch (e) {
            console.log("Failed to fetch sheet from cloud", year, monthStartWith1, e);
        }
    }

    // local
    let dbEntry = await db.months.where({year, month: monthStartWith1}).first();
    if (dbEntry) {
        if (dbEntry.created === undefined)
            dbEntry.created = Date.now();
        if (dbEntry.change_id === undefined)
            dbEntry.change_id = 1;
        return {cloud: false, month: fixMonth(dbEntry)};
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
    dbEntry = {days: dayEntries, year, month: monthStartWith1, created: Date.now(), change_id: 1};
    return {cloud: false, month: dbEntry};
}

export async function storeSheet(sheet: OneMonth): Promise<OneMonth> {
    if (sheet.created === undefined)
        sheet.created = Date.now();
    if (sheet.change_id === undefined)
        sheet.change_id = 1;
    else
        sheet.change_id += 1;
    await db.months.put(sheet);

    console.log("store", sheet);

    // cloud
    const s = get(cloudSettings);
    if (cloudUrl && s.cloud_api_key) {
        try {
            await post(cloudUrl + `/timesheets/${sheet.year}_${sheet.month}`, "application/json", JSON.stringify(sheet));
        } catch (e) {
            console.log("Failed to store sheet to cloud", sheet.year, sheet.month, e);
        }
    }

    return sheet;
}

interface ICSEntry {
    desc: string,
    uid: string,
    title: string,
    start: number,
    duration: number,
    confirmed: boolean,
    /// Out of office
    oof: boolean,
}

export async function fetchICS(monthIndex: number, day: number): Promise<ICSEntry[]> {
    const s = get(cloudSettings);
    if (s.cloud_url && s.cloud_api_key) {
        try {
            const response = await fetchWithTimeout(s.cloud_url + `/api/fetch_ics/${monthIndex + 1}/${day}`, {timeout: 12000});
            return await response.json();
        } catch (e) {
            console.log("Failed to fetch ICS", e);
        }
    }
    return [];
}

function toIso(date: Date): string {
    return date.toISOString().split('T')[0];
}

interface GitlabActivityEntryPushData {
    commit_title: string,
    action: string,
    ref_type: string,
    ref: string
}
interface GitlabActivityEntry {
    id: number,
    action_name: string,
    target_type: string|null,
    target_title: string|null,
    created_at: string,
    push_data?:GitlabActivityEntryPushData
}

export type {GitlabActivityEntry, GitlabActivityEntryPushData};

export async function fetchGitlabActivity(date: Date) :Promise<GitlabActivityEntry[]>{
    const s = get(localSettings);
    if (s.gitlab_url && s.gitlab_access_token) {
        try {
            const before = new Date(date);
            before.setDate(date.getDate() + 1);
            const after = new Date(date);
            after.setDate(date.getDate() - 1);
            const options = {timeout: 2000, no_auth: true, headers: {"PRIVATE-TOKEN": s.gitlab_access_token}};
            const response = await fetchWithTimeout(s.gitlab_url + `/api/v4/events?before=${toIso(before)}&after=${toIso(after)}`, options);
            return await response.json();
        } catch (e) {
            console.log("Failed to fetch gitlb activity", e);
        }
    }
    return [];
}

interface SelectedDate {
    date: number
}

export type {SelectedDate, ICSEntry};
export const selectedDate = writable<SelectedDate>({date: Date.now()});