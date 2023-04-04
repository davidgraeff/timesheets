<script lang="ts">
    import '@carbon/styles/css/styles.css';
    import '@carbon/charts/styles.css';
    import {BarChartSimple, PieChart} from "@carbon/charts-svelte";

    import {selectedDate} from "../../assets/js/data.js";
    import {localSettings} from "../../assets/js/settings.js";

    const zeroPad = (num, places = 2) => String(num).padStart(places, '0')

    function hh_mm(minutes: number) {
        return zeroPad(Math.round(minutes / 60)) + ":" + zeroPad(Math.round(minutes % 60));
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

    const exampleData: TimesheetPrintData = {
        header_date: "2022-12",
        period: "2022-12-01 to 2022-12-31",
        projects: [
            {
                name: "Agami",
                minutes: 124
            },
            {
                name: "Falco",
                minutes: 360
            }],
        total_minutes: 7380,
        tag_sums: [{name: "Meeting", minutes: 90}, {name: "Feature", minutes: 172}, {name: "Fix", minutes: 20}],
        weeks: [{
            period: "2022-03-01 to 2022-03-07",
            days: [
                {
                    date: new Date(2023, 3, 1), activities: [
                        {
                            project: "Agami",
                            minutes: 90,
                            tags: ["Meeting"],
                            description: "hisdf sidufbibas dazihu bdfai fbds\nfhsbdf ihsbdafuzwse sdf hb",
                        },
                        {
                            project: "Agami",
                            minutes: 80,
                            tags: ["Feature"],
                            description: "hisdf sidufbibas dazihu bdfai fbds",
                        },
                        {
                            project: "Agami",
                            minutes: 120,
                            tags: ["Fix"],
                            description: "hisdf sidufbibas dazihu bdfai fbds",
                        }
                    ]
                },
                {
                    date: new Date(2023, 3, 2), activities: [
                        {
                            project: "Agami",
                            minutes: 90,
                            tags: ["Meeting"],
                            description: "hisdf sidufbibas dazihu bdfai fbds\nfhsbdf ihsbdafuzwse sdf hb",
                        },
                        {
                            project: "Agami",
                            minutes: 80,
                            tags: ["Feature"],
                            description: "hisdf sidufbibas dazihu bdfai fbds",
                        },
                        {
                            project: "Agami",
                            minutes: 120,
                            tags: ["Fix"],
                            description: "hisdf sidufbibas dazihu bdfai fbds",
                        }
                    ]
                },
                {
                    date: new Date(2023, 3, 3), activities: [
                        {
                            project: "Agami",
                            minutes: 90,
                            tags: ["Meeting"],
                            description: "hisdf sidufbibas dazihu bdfai fbds\nfhsbdf ihsbdafuzwse sdf hb",
                        },
                        {
                            project: "Agami",
                            minutes: 80,
                            tags: ["Feature"],
                            description: "hisdf sidufbibas dazihu bdfai fbds",
                        },
                        {
                            project: "Agami",
                            minutes: 120,
                            tags: ["Fix"],
                            description: "hisdf sidufbibas dazihu bdfai fbds",
                        }
                    ]
                }
            ]
        }]
    }

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
                    const d = new Date(v.data.value * 1000);
                    return `${v.data.group} (${zeroPad(d.getHours().toString())}:${zeroPad(d.getMinutes().toString())}h)`;
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
        <h4 class="text-end">Week {week.period}</h4>
        <hr>
        <table class="table print-week-table">
            <tbody>
            {#each week.days as day}
                <tr>
                    <td><span>{weekday(day.date)}</span> {zeroPad(day.date.getDate())}.</td>
                    <td>
                        <table class="table table-striped print-day-table">
                            <tbody>
                                {#each day.activities as activity}
                                    <tr>
                                        <td>{activity.project}</td>
                                        <td>{hh_mm(activity.minutes)} h</td>
                                        <td>
                                            {#each activity.tags as tag}
                                                {tag}
                                            {/each}
                                        </td>
                                        <td>{@html activity.description.replace('\n', '<br>')}</td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </td>
                </tr>
            {/each}
            </tbody>
        </table>
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

    td:first-child, td:nth-child(1), td:nth-child(2) {
      white-space: nowrap;
    }

    margin-bottom: 0;
  }

  .print-week-table {
    width: auto;
    page-break-inside: avoid;

    > tbody > tr > td:first-child {
      font-weight: 700;
      --bs-text-opacity: 1;
      color: rgba(var(--bs-primary-rgb), var(--bs-text-opacity));

      white-space: nowrap;

      span {
        width: 35px;
        display: inline-block;
      }
    }

    td:last-child {
      width: 100%;
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