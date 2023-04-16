<script lang="ts">
    import {
        fetchGitlabActivity,
        fetchICS,
        loadSheet,
        selectedDate,
        storeSheet
    } from "../assets/js/data";
    import type {
        OneMonth,
        OneDay,
        DayEntry,
        SelectedDate,
        GitlabActivityEntry,
        ICSEntry
    } from "../assets/js/data";

    import {cloudSettings, localSettings} from "../assets/js/settings";
    import type {Settings} from "../assets/js/settings";

    import {onDestroy, onMount} from "svelte";
    import {shortcut} from "../assets/js/shortcut"
    import MultiSelect from 'svelte-multiselect'
    import {get} from "svelte/store";
    import {FormGroup, Input, Progress} from "sveltestrap";
    import {browser} from "$app/environment";
    import HoursMinutes from "../assets/components/HoursMinutes.svelte";
    import ConfirmDialog from "../assets/components/ConfirmDialog.svelte";

    // New entry
    let newProject = [];
    let newTags = [];
    let newDescription = "";
    let newDuration = 0;

    let newDurationEl: HoursMinutes | null = null;

    let entry: OneMonth = {year: 0, month: 0, days: [], created: 0, change_id: 0};

    const zeroPad = (num, places = 2) => String(num).padStart(places, '0')

    let currentDayDate = new Date();
    let currentDay: OneDay = {holiday: false, sick: false, expected_min_hours: 0, entries: []};
    let expected_min_hours = "00:00";

    function setExpectedMinHours(event: Event) {
        const target = event.target as HTMLInputElement;
        currentDay.expected_min_hours = Math.round(target.valueAsNumber / 1000 / 60 / 60);
        dayHasChanged();
    }

    const WEEK_DAY_NAMES = ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"];

    interface WeekDay {
        mapping: number,
        btnClasses: string
    }

    type Week = WeekDay[7];

    const EMPTY_WEEK: Week = [{mapping: 0, btnClasses: ""}, {mapping: 0, btnClasses: ""}, {mapping: 0, btnClasses: ""},
        {mapping: 0, btnClasses: ""}, {mapping: 0, btnClasses: ""}, {mapping: 0, btnClasses: ""}, {
            mapping: 0,
            btnClasses: ""
        }];

    let weeks: Week[] = [];
    const translateToMondayFirstDay = [6, 0, 1, 2, 3, 4, 5];

    function computeWeeks(date: Date) {
        const daysInMonth = new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();

        let localWeeks: Week[] = [];
        let currentWeek: Week = structuredClone(EMPTY_WEEK);

        let weekHasEntries = false;
        for (let i = 1; i <= daysInMonth; ++i) {
            const dayDate = new Date(date.getFullYear(), date.getMonth(), i);
            const dayOfWeek = translateToMondayFirstDay[dayDate.getDay()];
            currentWeek[dayOfWeek].mapping = i;
            currentWeek[dayOfWeek].btnClasses = "";
            weekHasEntries = true;
            if (dayOfWeek == 6) {
                localWeeks.push(currentWeek);
                currentWeek = structuredClone(EMPTY_WEEK);
                weekHasEntries = false;
            }
        }

        if (weekHasEntries)
            localWeeks.push(currentWeek);

        weeks = localWeeks;
    }

    function enableClass(clazz: string, classList: DOMTokenList, enable: boolean) {
        if (enable)
            classList.add(clazz);
        else
            classList.remove(clazz);
    }

    function sumActivities(activities: DayEntry[]) {
        let minutes = 0;
        for (let entry of activities)
            minutes += entry.duration;
        return minutes;
    }

    function applyStyleForDay(dayInMonth: number, localDate: Date, dayEntry: OneDay) {
        const day = localDate.getDate();
        const el = document.querySelector(`#btn-day-${dayInMonth}`);
        if (el) {
            if (dayInMonth == day)
                el.scrollIntoView({
                    behavior: 'smooth',
                    inline: 'center'
                });
            const sumMinutes = sumActivities(dayEntry.entries);
            const allEntries = dayEntry.holiday || dayEntry.sick || dayEntry.expected_min_hours * 60 <= sumMinutes;
            const dayDate = new Date(localDate.getFullYear(), localDate.getMonth(), dayInMonth);
            const dayOfWeek = translateToMondayFirstDay[dayDate.getDay()];
            enableClass("text-decoration-line-through", el.classList, dayEntry.holiday || dayEntry.sick);
            enableClass("text-decoration-underline", el.classList, dayEntry.expected_min_hours * 60 < sumMinutes && dayInMonth != day);
            enableClass("bg-success", el.classList, allEntries && dayOfWeek < 5 && dayInMonth != day);

            enableClass("border-danger", el.classList, dayInMonth == day);
            enableClass("border-warning", el.classList, sumMinutes === 0 && dayOfWeek < 5 && dayInMonth != day);
            enableClass("border-success", el.classList, sumMinutes > 0 && dayInMonth != day);
        } else {
            console.warn("Did not find element for", dayInMonth);
        }
    }

    //$: if (browser) applyStyleForDay(currentDayDate.getDate(), currentDayDate, currentDay);

    function setDay(localDate: Date) {
        if (!browser)
            return;

        currentDayDate = localDate;
        const day = localDate.getDate();
        if (entry.days.length < day)
            return;
        currentDay = entry.days[day - 1];
        expected_min_hours = zeroPad(currentDay.expected_min_hours) + ":00";

        // Update btn classes
        for (let dayInMonth = 1; dayInMonth <= entry.days.length; ++dayInMonth) {
            applyStyleForDay(dayInMonth, localDate, entry.days[dayInMonth - 1]);
        }

        if (newDurationEl)
            newDurationEl.focus();
    }


    let gitlabActivities: GitlabActivityEntry[] = [];
    let icsEntriesOfCurrentDay: string[] = [];

    function changeDay(dayIndex: number) {
        const d = new Date(get(selectedDate).date)
        d.setDate(dayIndex);
        gitlabActivities = [];
        icsEntriesOfCurrentDay = [];
        selectedDate.set({date: d.getTime()});
    }

    let focusCounter = 0;
    let cloudSynced = false;
    let activityChanges = false;

    async function refreshData(v: SelectedDate, forceReload = false) {
        if (activityChanges) return;

        const d = new Date(v.date);
        if (forceReload || entry.month != d.getMonth() || entry.year != d.getFullYear()) {
            const result = await loadSheet(d.getFullYear(), d.getMonth());
            cloudSynced = result.cloud;
            // Entry from DB/cloud is same as in-memory one
            if (entry.created != result.month.created) {
                entry = result.month;
                computeWeeks(d);
            }
            setTimeout(() => {
                setDay(d);
            }, 100);
        } else {
            setDay(d);
        }
    }

    async function pageGotFocus() {
        ++focusCounter;
        await refreshData(get(selectedDate), true);
    }

    onDestroy(() => {
        if (browser)
            document.removeEventListener("focus", pageGotFocus);
    })

    onMount(async () => {
        activityChanges = false;
        if (browser)
            document.addEventListener("focus", pageGotFocus);
        return selectedDate.subscribe(refreshData);
    })

    function dayHasChanged() {
        activityChanges = true;
    }

    async function saveActivity(e) {
        if (e)
            e.preventDefault();

        if (newDescription.length !== 0) {
            const dayEntry: DayEntry = {
                description: newDescription,
                duration: newDuration,
                import_tags: [],
                project: newProject,
                tags: newTags
            };
            currentDay.entries.push(dayEntry);
            await saveDay();
            currentDay.entries = currentDay.entries;
            newDescription = "";
            newTags = [];
            newProject = [];
        } else if (activityChanges)
            await saveDay();
        setTimeout(() => {
            if (newDurationEl)
                newDurationEl.focus();
        }, 100);
    }

    async function nextDay() {
        await saveActivity(null);
        const d = new Date(currentDayDate.getTime());
        d.setDate(d.getDate() + 1);
        setDay(d);
    }

    async function removeActivity(activityIndex: number) {
        currentDay.entries.splice(activityIndex, 1);
        currentDay.entries = currentDay.entries;
        await saveDay();
    }

    async function saveDay() {
        entry = await storeSheet(entry);
        activityChanges = false;
    }

    function computeProgress(cDay: OneDay) {
        let minutes = 0;
        for (let activity of cDay.entries)
            minutes += activity.duration;
        return minutes * 100 / (cDay.expected_min_hours * 60);
    }

    let loadingICS = false;


    async function addFromICS() {
        loadingICS = true;
        const icsEntries: ICSEntry[] = await fetchICS(currentDayDate.getMonth(), currentDayDate.getDate());


        const knownUids = new Set<string>()
        for (let entry of currentDay.entries) {
            for (let uid of entry.import_tags)
                knownUids.add(uid);
        }

        let icsEntriesCurrentDayLocal = [];
        for (let entry of icsEntries) {
            icsEntriesCurrentDayLocal.push(`${hh_mm(entry.duration / 60)}: ${entry.title.replaceAll("\\n", "\n")}`);

            if (knownUids.has(entry.uid))
                continue;
            const dayEntry: DayEntry = {
                description: entry.title.replaceAll("\\n", "\n"), //  + "\n" + entry.desc.replaceAll("\\n", "\n")
                duration: entry.duration / 60,
                import_tags: [entry.uid],
                project: ["Agami"],
                tags: []
            };
            if (entry.oof) dayEntry.description = "OOF " + dayEntry.description;
            if (!entry.confirmed) dayEntry.description = "NOT CONFIRMED " + dayEntry.description;
            dayEntry.description = dayEntry.description.trim();
            currentDay.entries.push(dayEntry);
        }

        icsEntriesOfCurrentDay = icsEntriesCurrentDayLocal;

        await saveDay();
        currentDay.entries = currentDay.entries;

        loadingICS = false;
    }

    let loadingGitlabActivity = false;

    function hh_mm(minutes: number) {
        return zeroPad(Math.floor(minutes / 60)) + ":" + zeroPad(Math.round(minutes % 60));
    }

    async function showGitlabActivity() {
        loadingGitlabActivity = true;
        gitlabActivities = await fetchGitlabActivity(currentDayDate);
        console.log("gitlab activity entries", gitlabActivities);

        loadingGitlabActivity = false;
    }

    async function clearDay() {
        currentDay.entries = [];
        await saveDay();
    }

    let confirmDialog: ConfirmDialog | null = null;

    function shortDate(created_at: string) {
        const d = new Date(created_at)
        return `${zeroPad(d.getMonth() + 1)}-${zeroPad(d.getDate())} ${zeroPad(d.getHours())}:${zeroPad(d.getMinutes())}`
    }

    function missingTime(currentDay: OneDay) {
        const current = sumActivities(currentDay.entries);
        const remaining = currentDay.expected_min_hours * 60 - current;
        return `Current: ${hh_mm(current)}. Remaining: ${hh_mm(remaining)}`;
    }

</script>

<ConfirmDialog desc="Remove all entries of this day?" bind:this={confirmDialog}></ConfirmDialog>

<div class="container container-lg">
    <div class="d-flex justify-content-start overflow-scroll gap-2">
        {#each weeks as week, week_index}
            <table class="table-responsive table-bordered table-weeks mb-2">
                <thead>
                <tr>
                    {#each week as week_day, day_index}
                        <td class:text-muted={week_day.mapping===0 || day_index>4}>{WEEK_DAY_NAMES[day_index]}</td>
                    {/each}
                </tr>
                </thead>
                <tbody>
                <tr>
                    {#each week as week_day}
                        <td>
                            {#if week_day.mapping !== 0}
                                <button type="button"
                                        class="btn btn-sm w-100 border border-2"
                                        id="btn-day-{week_day.mapping}"
                                        disabled={activityChanges}
                                        on:click={() => changeDay(week_day.mapping)}>{week_day.mapping}</button>
                            {/if}
                        </td>
                    {/each}
                </tr>
                </tbody>
            </table>
        {/each}
    </div>

    <div class="my-2 d-flex justify-content-end gap-2">
        <Progress class="w-100" max="100" value={computeProgress(currentDay)}></Progress>
        {#if cloudSynced}
            <div>Cloud sync {entry.change_id}</div>
        {/if}
        <div>
            <div class="form-check form-switch">
                <input class="form-check-input" type="checkbox" id="switchHoliday" bind:checked={currentDay.holiday}
                       on:blur={saveDay}
                       use:shortcut={{alt: true, code: 'KeyH'}}>
                <label class="form-check-label text-nowrap" for="switchHoliday">Holiday (H)</label>
            </div>
            <div class="form-check form-switch">
                <input class="form-check-input" type="checkbox" id="switchSick" bind:checked={currentDay.sick}
                       on:blur={saveDay}
                       use:shortcut={{alt: true, code: 'KeyI'}}>
                <label class="form-check-label text-nowrap" for="switchSick">Ill/Sick (I)</label>
            </div>
        </div>
        <div class="mb-3 form-floating">
            <input class="form-control" style="min-width: 130px" placeholder="Enter a value" type="time"
                   on:change={setExpectedMinHours} bind:value={expected_min_hours} id="input_min_hours"/>
            <label use:shortcut={{alt: true, code: 'KeyM'}} for="input_min_hours">Min hours (M) </label>
        </div>
    </div>

    <table class="table table-borderless">
        <thead>
        <tr class="border-bottom">
            <td style="width: 10px"></td>
            <td style="width: 100px">Duration</td>
            <td style="width: 120px">Project</td>
            <td>Tags</td>
        </tr>
        </thead>
        <tbody>
        {#each currentDay.entries as dayEntry, activityIndex}
            <tr>
                <td>
                    <button type="button" class="btn btn-close"
                            use:shortcut={{alt: true, code: 'KeyN'}}
                            on:click={() => removeActivity(activityIndex)}>
                    </button>
                </td>
                <td>
                    <HoursMinutes bind:value={dayEntry.duration} on:change={dayHasChanged}></HoursMinutes>
                </td>
                <td class="small-dropdown">
                    <MultiSelect bind:selected={dayEntry.project} on:change={dayHasChanged} maxSelect={1} minSelect={1}
                                 options={$localSettings.projects}/>
                </td>
                <td class="small-dropdown-lg">
                    <MultiSelect bind:selected={dayEntry.tags} on:change={dayHasChanged} options={$localSettings.tags}/>
                </td>
            </tr>
            <tr class="border-bottom">
                <td></td>
                <td colspan="3">
                  <textarea bind:value={dayEntry.description} on:change={dayHasChanged} rows="1" class="form-control"
                            placeholder="Descriptive text..."></textarea>
                </td>
            </tr>
        {/each}
        <tr>
            <td colspan="4">
                <h3>New entry ({missingTime(currentDay)})</h3></td>
        </tr>
        <tr>
            <td></td>
            <td>
                <HoursMinutes bind:this={newDurationEl} bind:value={newDuration}></HoursMinutes>
            </td>
            <td class="small-dropdown">
                <MultiSelect bind:selected={newProject} maxSelect={1} minSelect={1} options={$localSettings.projects}/>
            </td>
            <td class="small-dropdown-lg">
                <MultiSelect bind:selected={newTags} options={$localSettings.tags}/>
            </td>
        </tr>
        <tr>
            <td></td>
            <td colspan="3">
                <textarea bind:value={newDescription} on:blur={saveActivity} rows="3" class="form-control"
                          placeholder="Descriptive text..."></textarea>
            </td>
        </tr>
        </tbody>
    </table>

    <div class="d-flex justify-content-end">
        <div class="btn-group">

            <button type="button" class="btn btn-primary"
                    use:shortcut={{alt: true, code: 'KeyN'}}
                    on:click={nextDay}>Next day (N)
            </button>

            {#if activityChanges}
                <button type="button" class="btn btn-primary"
                        use:shortcut={{alt: true, code: 'KeyS'}}
                        on:click={saveDay}>Save day (S)
                </button>
            {/if}

            <button type="button" class="btn btn-danger" on:click={() => confirmDialog.open(clearDay)}>Clear day
            </button>
            <button type="button" class="btn btn-secondary" on:click={addFromICS} disabled={loadingICS}>Add entries from
                ICS
            </button>
            <button type="button" class="btn btn-secondary" on:click={showGitlabActivity}
                    disabled={loadingGitlabActivity}>
                Show Gitlab Activity
            </button>

        </div>
    </div>

    {#if icsEntriesOfCurrentDay.length > 0}
        <ul>
            {#each icsEntriesOfCurrentDay as icsEntry}
                <li>{icsEntry}</li>
            {/each}
        </ul>
    {/if}

    {#if gitlabActivities.length > 0}
        <table class="table mt-3">
            <thead>
            <tr class="border-bottom">
                <td style="width: 100px">Date</td>
                <td style="width: 200px">Action</td>
                <td>Title</td>
            </tr>
            </thead>
            <tbody>
            {#each gitlabActivities as activity, activityIndex}
                <tr>
                    <td>{shortDate(activity.created_at)}</td>
                    <td>
                        {#if activity.push_data}
                            {activity.push_data.action}
                        {:else }
                            {activity.action_name}
                        {/if}
                        {#if activity.target_type}
                            {activity.target_type}
                        {/if}
                    </td>
                    <td>
                        {#if activity.target_title}
                            {activity.target_title}
                        {/if}
                        {#if activity.push_data && activity.push_data.commit_title}
                            {activity.push_data.commit_title}
                        {/if}
                        {#if activity.push_data && activity.push_data.ref}
                            {activity.push_data.ref_type} {activity.push_data.ref}
                        {/if}
                    </td>
                </tr>
            {/each}
            </tbody>
        </table>
    {/if}
</div>