<script lang="ts">
    import '@carbon/styles/css/styles.css';
    import '@carbon/charts/styles.css';
    import {BarChartSimple, PieChart} from "@carbon/charts-svelte";
    import {marked} from 'marked';

    import type {SelectedDate, OneMonth} from "../../assets/js/data.js";

    import {onMount} from 'svelte';
    import {selectedDate, loadSheet} from "../../assets/js/data.js";
    import {localSettings} from "../../assets/js/settings.js";

    const zeroPad = (num, places = 2) => String(num).padStart(places, '0')

    function hh_mm(minutes: number) {
        return zeroPad(Math.floor(minutes / 60)) + ":" + zeroPad(Math.round(minutes % 60));
    }

    interface ProjectHours {
        name: string,
        minutes: number
    }

    interface ProjectActivity {
        project: string,
        minutes: number,
        tags: string[],
        description: string
    }

    interface ProjectDays {
        date: Date,
        activities: ProjectActivity[]
    }

    interface ProjectWeek {
        period: string,
        days: ProjectDays[],
    }

    interface TimesheetPrintData {
        header_date: string,
        period: string,
        total_minutes: number,
        projects: ProjectHours[],
        weeks: ProjectWeek[]
        tag_sums: ProjectHours[],
    }

    let exampleData: TimesheetPrintData = {
        header_date: "",
        period: "",
        projects: [],
        tag_sums: [],
        total_minutes: 0,
        weeks: []
    };

    const project_chart_options = {
        height: "170px",
        width: "320px",
        resizable: false,
        toolbar: {enabled: false},
        pie: {
            alignment: "left",
            labels: {
                enabled: true,
                formatter: v => {
                    return `${v.data.group} (${hh_mm(v.data.value)}h)`;
                }
            }
        },
        legend: {
            enabled: false,
            clickable: false,
            alignment: "center",
            position: "right",
            orientation: "vertical",
            truncation: {numCharacter: 100}
        }
    };


    function projectHoursChartData(data: ProjectHours[]) {
        let entries = [];
        for (const project of data)
            entries.push({group: project.name, value: project.minutes});

        return entries;
    }

    function weekday(date: Date) {
        return date.toLocaleDateString('en-US', {weekday: 'short'});
    }

    function computeWeeks(date: Date) {
        const daysInMonth = new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
        const translateToMondayFirstDay = [6, 0, 1, 2, 3, 4, 5];

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

    async function refreshData(v: SelectedDate) {
        const d = new Date(v.date);
        const dCopy = new Date(d.getFullYear(), d.getMonth() + 1, 0);
        const result = await loadSheet(d.getFullYear(), d.getMonth());
        const entry: OneMonth = result.month;
        let newPageData: TimesheetPrintData = {
            header_date: `${entry.year}-${zeroPad(entry.month)}`,
            period: `${entry.year}-${zeroPad(entry.month)}-01 to ${entry.year}-${zeroPad(entry.month)}-${zeroPad(dCopy.getDate())}`,
            projects: [],
            tag_sums: [],
            total_minutes: 0,
            weeks: []
        };

        let tags = new Map<string, number>();
        let projects = new Map<string, number>();
        const translateToMondayFirstDay = [6, 0, 1, 2, 3, 4, 5];

        let currentWeek: ProjectWeek = {days: [], period: ""};
        let weekHasEntries = false;

        let dayOfMonth = 1;
        for (let day of entry.days) {
            const dayDate = new Date(entry.year, entry.month - 1, dayOfMonth);
            const dayOfWeek = translateToMondayFirstDay[dayDate.getDay()];

            let projectDay: ProjectDays = {activities: [], date: dayDate};
            for (let entry of day.entries) {
                let activity: ProjectActivity = {
                    description: marked.parse(entry.description),
                    minutes: entry.duration,
                    project: "",
                    tags: entry.tags
                };
                newPageData.total_minutes += entry.duration;
                for (let tagName of entry.tags) {
                    let existingEntry = tags.get(tagName);
                    if (existingEntry)
                        tags.set(tagName, entry.duration + existingEntry);
                    else
                        tags.set(tagName, entry.duration);
                }
                for (let projectName of entry.project) {
                    activity.project = projectName;
                    let existingEntry = projects.get(projectName);
                    if (existingEntry)
                        projects.set(projectName, entry.duration + existingEntry);
                    else
                        projects.set(projectName, entry.duration);
                }
                projectDay.activities.push(activity);
            }

            if (projectDay.activities.length === 0) {
                if (dayOfWeek < 5) {
                    let activity: ProjectActivity = {
                        description: "-",
                        minutes: 0,
                        project: "",
                        tags: [""]
                    };
                    if (day.holiday) activity.description = "-Holiday-";
                    if (day.sick) activity.description = "-Sick-";
                    projectDay.activities.push(activity);
                }
            }

            if (projectDay.activities.length > 0)
                currentWeek.days.push(projectDay);
            weekHasEntries = true;

            if (dayOfWeek == 6) {
                currentWeek.period = weekPeriodString(entry, dayOfMonth);
                newPageData.weeks.push(currentWeek);
                currentWeek = {days: [], period: ""};
                weekHasEntries = false;
            }

            ++dayOfMonth;
        }

        if (weekHasEntries) {
            currentWeek.period = weekPeriodString(entry, dayOfMonth - 1);
            newPageData.weeks.push(currentWeek);
        }

        for (let [tag, duration] of tags) {
            newPageData.tag_sums.push({name: tag, minutes: duration});
        }

        newPageData.tag_sums = newPageData.tag_sums.sort((a: ProjectHours, b: ProjectHours) => {
            const x = a.minutes;
            const y = b.minutes;
            return ((x < y) ? 1 : ((x > y) ? -1 : 0));
        });
        for (let [tag, duration] of projects) {
            newPageData.projects.push({name: tag, minutes: duration});
        }

        exampleData = newPageData;
    }

    function weekPeriodString(entry: OneMonth, lastDay: number) {
        const firstDay = Math.max(1, lastDay - 6);
        lastDay = Math.min(lastDay, entry.days.length + 1);
        return `${entry.year}-${zeroPad(entry.month)}-${zeroPad(firstDay)} to ${entry.year}-${zeroPad(entry.month)}-${zeroPad(lastDay)}`;
    }

    onMount(() => {
        return selectedDate.subscribe(refreshData);
    })

    function withComma(tag, index: number, length: number) {
        if (index + 1 < length)
            return tag + ",";
        else
            return tag;
    }

    function allTags(tags: string[]) {
        let result = "";
        for (let index = 0; index < tags.length; ++index) {
            const tag = tags[index];
            result += withComma(tag, index, tags.length) + " ";
        }
        return result;
    }

    function sumActivities(activities: ProjectActivity[]) {
        let total = 0;
        for (let activity of activities)
            total += activity.minutes;
        return total;
    }

</script>

<div class="container" style="width: 100%;height: 100%;background-color: white">

    <h1>Timesheet {$localSettings.name} - {exampleData.header_date}</h1>
    <div class="mt-2 d-flex gap-3">

        <table class="table table-light table-striped" style="height: fit-content;">
            <tbody>
            <tr>
                <td>Client</td>
                <td style="text-align: right">{$localSettings.client}</td>
            </tr>
            <tr>
                <td>Worker</td>
                <td style="text-align: right">{$localSettings.name}</td>
            </tr>
            </tbody>
        </table>
        <table class="table table-light table-striped" style="height: fit-content;">
            <tbody>
            <tr>
                <td>Agency</td>
                <td style="text-align: right">{$localSettings.company}</td>
            </tr>
            <tr>
                <td>Period</td>
                <td style="text-align: right">{exampleData.period}</td>
            </tr>
            </tbody>
        </table>
    </div>


    <div class="d-flex float-end">
        <div class="border border-1 p-2 mt-2 mb-3 ms-auto signature-container">
            <div class="border border-1 border-dark"></div>
            <div class="p2">Signature</div>
        </div>
    </div>

    <table class="table table-striped border" style="width: 230px; height: fit-content;">
        <thead>
        <tr class="text-primary fw-bold">
            <th style="width: 120px">Total hours</th>
            <th class="text-end text-nowrap">{hh_mm(exampleData.total_minutes)} h</th>
        </tr>
        </thead>
        <tbody>
        {#each exampleData.projects as project}
            <tr>
                <td>{project.name}</td>
                <td class="text-end text-nowrap">{hh_mm(project.minutes)} h</td>
            </tr>
        {/each}
        </tbody>
    </table>

    <div class="clearfix"></div>
    {#each exampleData.weeks as week}
        <div class="avoid-page-break mb-2">
            <h4 class="text-end pt-2">Week {week.period}</h4>
            <hr>
            {#each week.days as day}
                <div class="avoid-page-break">
                    <div class="week-day">
                        <span>{weekday(day.date)}</span> {zeroPad(day.date.getDate())}
                        ({hh_mm(sumActivities(day.activities))})
                    </div>
                    <table class="table table-striped print-day-table">
                        <tbody>
                        {#each day.activities as activity}
                            <tr>
                                <td>{activity.project}</td>
                                <td>{hh_mm(activity.minutes)} h</td>
                                <td class="ptag">
                                    {allTags(activity.tags)}
                                    <!--{#each activity.tags as tag, index}-->
                                    <!--    {withComma(tag, index, activity.tags.length)}&nbsp;-->
                                    <!--{/each}-->
                                </td>
                                <td>{@html activity.description}</td>
                            </tr>
                        {/each}
                        </tbody>
                    </table>
                </div>
            {/each}
        </div>
    {/each}

    <section style="page-break-inside: avoid;">
        <h4 class="text-end">Statistics</h4>
        <hr>
        <div class="d-flex gap-2 align-items-center justify-content-center flex-nowrap">
            <PieChart
                    data={projectHoursChartData(exampleData.projects)}
                    options={project_chart_options}
            />
            <PieChart
                    data={projectHoursChartData(exampleData.tag_sums)}
                    options={project_chart_options}
            />
            <table class="table table-striped" style="height: fit-content;width: fit-content">
                <tbody>
                {#each exampleData.tag_sums as project}
                    <tr>
                        <td>{project.name}</td>
                        <td class="text-end text-nowrap">{hh_mm(project.minutes)} h</td>
                    </tr>
                {/each}
                </tbody>
            </table>
        </div>
    </section>
</div>

<style lang="scss">
  .print-day-table {
    page-break-inside: avoid;

    td:first-child {
      white-space: nowrap;
      width: 70px;
    }

    td:nth-child(2) {
      white-space: nowrap;
      width: 80px;
    }

    td:nth-child(3) {
      width: 130px;
    }

    td:nth-child(4){
      :global(p:last-child) {
        margin-bottom: 0;
      }

      :global(ul) {
        list-style: circle;
      }
    }

    margin-bottom: 0;
  }

  .avoid-page-break {
    page-break-inside: avoid;
  }

  .week-day {
    font-weight: 700;
    --bs-text-opacity: 1;
    color: rgba(var(--bs-primary-rgb), var(--bs-text-opacity));

    white-space: nowrap;
    margin-bottom: 10px;
    margin-top: 10px;

    span {
      width: 35px;
      display: inline-block;
    }
  }

  .signature-container {
    height: 100px;
    width: 400px;
    position: relative;

    & > div:first-child {
      position: absolute;
      bottom: 26px;
      left: 5px;
      right: 5px;
    }

    & > div:nth-child(2) {
      position: absolute;
      right: 5px;
      bottom: 5px;
    }

  }
</style>