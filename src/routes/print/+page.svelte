<script lang="ts">
    import '@carbon/styles/css/styles.css';
    import '@carbon/charts/styles.css';
    import {BarChartSimple, PieChart} from "@carbon/charts-svelte";

    import {selectedDate} from "../../assets/js/data.js";
    import {localSettings} from "../../assets/js/settings.js";

    let header_date = "2022-12";
    let period = "2022-12-01 to 2022-12-31"
    let project_chart_options = {
        height: "200px",
        width: "300px",
        resizable: false,
        toolbar: {enabled: false},
        pie: {
            alignment: "center",
            labels: {
                enabled: true, formatter: v => {
                    const d = new Date(v.data.value * 1000);
                    return `${v.data.group} (${d.getHours()}:${d.getMinutes()}h)`;
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
    let project_chart_data = [
        {
            "group": "Project 1",
            "value": 20000
        },
        {
            "group": "Agami",
            "value": 65000
        },
        {
            "group": "Tukan",
            "value": 75000
        },
    ];
</script>

<div class="container" style="width: 100%;height: 100%;background-color: white">

    <h1 class="py-3 ps-3">Timesheet {$localSettings.name} - {header_date}</h1>
    <div class="p-3 mt-2 d-flex gap-3">

        <table class="table table-light table-striped" style="max-width: 300px">
            <tbody>
            <tr>
                <td>Client</td>
                <td style="text-align: right">{$localSettings.client}</td>
            </tr>
            <tr>
                <td>Worker</td>
                <td style="text-align: right">{$localSettings.name}</td>
            </tr>
            <tr>
                <td>Agency</td>
                <td style="text-align: right">{$localSettings.company}</td>
            </tr>
            <tr>
                <td>Period</td>
                <td style="text-align: right">{period}</td>
            </tr>
            </tbody>
        </table>
        <div style="flex-grow: 1;">&nbsp;</div>
        <table class="table table-striped border" style="width: 230px">
            <thead>
            <tr class="text-primary fw-bold">
                <th style="width: 120px">Total hours</th>
                <th class="text-end text-nowrap">123:00 h</th>
            </tr>
            </thead>
            <tbody>
            <tr>
                <td>Project 1</td>
                <td class="text-end text-nowrap">34:00 h</td>
            </tr>
            <tr>
                <td>Project 1</td>
                <td class="text-end text-nowrap">3:00 h</td>
            </tr>
            <tr>
                <td>Project 1</td>
                <td class="text-end text-nowrap">64:00 h</td>
            </tr>
            </tbody>
        </table>
        <PieChart
                data={project_chart_data}
                options={project_chart_options}
        />
    </div>

    <div class="d-flex">
        <div class="border border-1 p-2 mt-2 mb-3 ms-auto"
             style="height: 100px; width: 400px; display: flex; flex-direction: column; justify-content: end;align-items: end;">
            <div class="border border-1 w-100"></div>
            <div class="p2">Signature</div>
        </div>
    </div>

    <h4 class="text-end">Week 2022-03-01 to 2022-03-07</h4>
    <hr>
    <table class="table print-week-table">
        <tbody>
        <tr>
            <td><span>Mo</span> 01.</td>
            <td>
                <table class="table table-striped print-day-table">
                    <tbody>
                    <tr>
                        <td>Agami</td>
                        <td>01:30 h</td>
                        <td>Meeting</td>
                        <td>hisdf sidufbibas dazihu bdfai fbds<br> fhsbdf ihsbdafuzwse sdf hb</td>
                    </tr>
                    <tr>
                        <td>Agami</td>
                        <td>01:30 h</td>
                        <td>Feature</td>
                        <td>hisdf sidufbibas dazihu bdfai fbds fhsbdf ihsbdafuzwse sdf hb</td>
                    </tr>
                    <tr>
                        <td>Agami</td>
                        <td>01:30 h</td>
                        <td>Fix</td>
                        <td>hisdf sidufbibas dazihu bdfai fbds fhsbdf ihsbdafuzwse sdf hb</td>
                    </tr>
                    </tbody>
                </table>
            </td>
        </tr>
        <tr>
            <td><span>Di</span> 02.</td>
            <td>
                <table class="table table-striped print-day-table">
                    <tbody>
                    <tr>
                        <td>Agami</td>
                        <td>01:30 h</td>
                        <td>Meeting</td>
                        <td>hisdf sidufbibas dazihu bdfai fbds<br> fhsbdf ihsbdafuzwse sdf hb</td>
                    </tr>
                    <tr>
                        <td>Agami</td>
                        <td>01:30 h</td>
                        <td>Feature</td>
                        <td>hisdf sidufbibas dazihu bdfai fbds fhsbdf ihsbdafuzwse sdf hb</td>
                    </tr>
                    <tr>
                        <td>Agami</td>
                        <td>01:30 h</td>
                        <td>Fix</td>
                        <td>hisdf sidufbibas dazihu bdfai fbds fhsbdf ihsbdafuzwse sdf hb</td>
                    </tr>
                    </tbody>
                </table>
            </td>
        </tr>
        <tr>
            <td><span>Mi</span> 03.</td>
            <td>
                <table class="table table-striped print-day-table">
                    <tbody>
                    <tr>
                        <td>Agami</td>
                        <td>01:30 h</td>
                        <td>Meeting</td>
                        <td>hisdf sidufbibas dazihu bdfai fbds<br> fhsbdf ihsbdafuzwse sdf hb</td>
                    </tr>
                    <tr>
                        <td>Agami</td>
                        <td>01:30 h</td>
                        <td>Feature</td>
                        <td>hisdf sidufbibas dazihu bdfai fbds fhsbdf ihsbdafuzwse sdf hb</td>
                    </tr>
                    <tr>
                        <td>Agami</td>
                        <td>01:30 h</td>
                        <td>Fix</td>
                        <td>hisdf sidufbibas dazihu bdfai fbds fhsbdf ihsbdafuzwse sdf hb</td>
                    </tr>
                    </tbody>
                </table>
            </td>
        </tr>
        </tbody>
    </table>

    <section style="page-break-inside: avoid;">
        <h4 class="text-end">Statistics</h4>
        <hr>
        <div class="d-flex">
            <table class="table table-striped ms-auto" style="width: 300px;">
                <tbody>
                <tr>
                    <td>Meeting</td>
                    <td>01:30 h</td>
                </tr>
                <tr>
                    <td>Feature</td>
                    <td>01:30 h</td>
                </tr>
                <tr>
                    <td>Fix</td>
                    <td>01:30 h</td>
                </tr>
                </tbody>
            </table>
        </div>
    </section>
</div>

<style lang="scss">
  .print-day-table {
    page-break-inside: avoid;
  }

  .print-week-table {
    width: auto;
    border-spacing: 5px;
    border-collapse: separate;
    page-break-inside: avoid;

    td {
      white-space: nowrap;
    }

    > tbody > tr > td:first-child {
      font-weight: 700;
      --bs-text-opacity: 1;
      color: rgba(var(--bs-primary-rgb), var(--bs-text-opacity));

      white-space: nowrap;

      span {
        width: 25px;
        display: inline-block;
      }
    }

    td:last-child {
      width: 100%;
    }
  }
</style>