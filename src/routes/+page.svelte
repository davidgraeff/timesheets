<script lang="ts">
    import {loadSheet, selectedDate, storeSheet} from "../assets/js/data";
    import type {OneMonth, OneDay, DayEntry, SelectedDate} from "../assets/js/data";

    import {cloudSettings, localSettings} from "../assets/js/settings";
    import type {Settings} from "../assets/js/settings";

    import {onDestroy, onMount} from "svelte";
    import {shortcut} from "../assets/js/shortcut"
    import MultiSelect from 'svelte-multiselect'
    import {get} from "svelte/store";
    import {FormGroup, Input} from "sveltestrap";
    import {browser} from "$app/environment";

    // New entry
    let newProject = "";
    let newTags = [];
    let newDescription;
    let newDuration;

    let entry: OneMonth = {year: 0, month: 0, days: [], created: 0, change_id: 0};

    const zeroPad = (num, places = 2) => String(num).padStart(places, '0')

    let currentDayDate = new Date();
    let currentDay: OneDay = {holiday: false, sick: false, expected_min_hours: 0, entries: []};
    let expected_min_hours = "00:00";

    function setExpectedMinHours(event: Event) {
        const target = event.target as HTMLInputElement;
        currentDay.expected_min_hours = Math.round(target.valueAsNumber / 1000 / 60 / 60);
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
            enableClass("border-danger", el.classList, dayInMonth == day);
            enableClass("text-decoration-line-through", el.classList, dayEntry.holiday || dayEntry.sick);
            const sumMinutes = sumActivities(dayEntry.entries);
            const allEntries = dayEntry.holiday || dayEntry.sick || dayEntry.expected_min_hours * 60 < sumMinutes;
            const dayDate = new Date(localDate.getFullYear(), localDate.getMonth(), dayInMonth);
            const dayOfWeek = translateToMondayFirstDay[dayDate.getDay()];
            enableClass("bg-success", el.classList, allEntries && dayOfWeek < 5);
            enableClass("border-warning", el.classList, sumMinutes === 0 && dayOfWeek < 5 && dayInMonth != day);
            enableClass("border-success", el.classList, sumMinutes > 0 && dayInMonth != day);
        }
    }

    $: if (browser) applyStyleForDay(currentDayDate.getDate(), currentDayDate, currentDay);

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

        if (newDuration)
            newDuration.focus();
    }

    function changeDay(dayIndex: number) {
        const d = new Date(get(selectedDate).date)
        d.setDate(dayIndex);
        selectedDate.set({date: d.getTime()});
    }

    let focusCounter = 0;

    async function refreshData(v: SelectedDate) {
        const d = new Date(v.date);
        if (entry.month != d.getMonth() || entry.year != d.getFullYear()) {
            const result = await loadSheet(d.getFullYear(), d.getMonth());
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
        await refreshData(get(selectedDate));
    }

    onDestroy(() => {
        if (browser)
            document.removeEventListener("focus", pageGotFocus);
    })

    onMount(async () => {
        if (browser)
            document.addEventListener("focus", pageGotFocus);
        return selectedDate.subscribe(refreshData);
    })

    async function saveEntry(e) {
        e.preventDefault();
        const descParts = newDescription.value.split('\n');
        const title = descParts[0];
        const description = descParts.length > 1 ? descParts.splice(1).join("\n") : "";

        if (title.length !== 0) {
            const duration = newDuration.valueAsNumber / 1000;
            currentDay.entries.push({
                description,
                duration,
                import_tags: [],
                project: newProject,
                tags: newTags,
                title
            });
            await storeSheet(entry);
            currentDay.entries = currentDay.entries;
            if (newDescription)
                newDescription.value = "";
            newTags = [];
        }
        setTimeout(() => {
            if (newDuration)
                newDuration.focus();
        }, 100);
    }

    function nextDay() {
        const d = new Date(currentDayDate.getTime());
        d.setDate(d.getDate() + 1);
        setDay(d);
    }

</script>

<div>focus {focusCounter}</div>

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
    <div>
        <div class="form-check form-switch">
            <input class="form-check-input" type="checkbox" id="switchHoliday" bind:checked={currentDay.holiday}
                   use:shortcut={{alt: true, code: 'KeyH'}}>
            <label class="form-check-label" for="switchHoliday">Holiday (H)</label>
        </div>
        <div class="form-check form-switch">
            <input class="form-check-input" type="checkbox" id="switchSick" bind:checked={currentDay.sick}
                   use:shortcut={{alt: true, code: 'KeyS'}}>
            <label class="form-check-label" for="switchSick">Sick (S)</label>
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
        <td style="width: 100px">Duration</td>
        <td style="width: 120px">Project</td>
        <td>Tags</td>
        <td></td>
    </tr>
    </thead>
    <tbody>
    {#each currentDay.entries as dayEntry}
        <div>Entry {dayEntry.duration}  {dayEntry.tags}  {dayEntry.project}  {dayEntry.title}</div>
    {/each}
    <tr>
        <td>
            <input bind:this={newDuration} type="time" max="08:00:00" min="00:00:00" value="01:00"
                   pattern="0[0-8]:[0-9][0-9]" class="form-control"/>
        </td>
        <td class="small-dropdown">
            <MultiSelect bind:value={newProject} maxSelect={1} minSelect={1} options={$localSettings.projects}/>
        </td>
        <td class="small-dropdown-lg">
            <MultiSelect bind:value={newTags} options={$localSettings.tags}/>
        </td>
    </tr>
    <tr class="border-bottom">
        <td colspan="4">
            <div class="d-flex gap-2">
                <textarea bind:this={newDescription} on:blur={saveEntry} rows="3" class="form-control"
                          placeholder="Descriptive text..."></textarea>
                <!--                <button type="button" class="btn btn-danger" tabindex="-1"-->
                <!--                        on:click={() => removeEntry(dayEntryIndex)}>X-->
                <!--                </button>-->
            </div>
        </td>
    </tr>
    </tbody>
</table>

<button type="button" class="btn btn-primary"
        use:shortcut={{alt: true, code: 'KeyN'}}
        on:click={nextDay}>Next day (N)
</button>