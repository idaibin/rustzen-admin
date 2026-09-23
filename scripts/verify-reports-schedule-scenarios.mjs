export const SCHEDULE_WAIT_ATTEMPTS = 360;
export const SCHEDULE_WAIT_MS = 250;

export async function waitForScheduleDecision({ directRequest, expectStatus, responseData, reportsBase, id, decision, sleep }) {
    const path = `/api/reports/schedules/${id}`;
    for (let attempt = 0; attempt < SCHEDULE_WAIT_ATTEMPTS; attempt += 1) {
        const current = await responseData(
            await expectStatus(await directRequest(reportsBase, "reports", path, "reports:schedule:view"), 200, `read ${decision} schedule`),
            `read ${decision} schedule`,
        );
        if (current.lastOccurrence?.decision === decision) return current;
        await sleep(SCHEDULE_WAIT_MS);
    }
    throw new Error(`schedule ${id} did not record ${decision} within 90 seconds`);
}

export async function verifyReportsScheduleScenarios({
    directRequest,
    expectStatus,
    responseData,
    reportsBase,
    reportsRuntimeRoot,
    flow,
    now,
    sleep,
    spawnSync,
}) {
    const reportsDatabase = `${reportsRuntimeRoot}/data/reports/db/reports.db`;
    const scheduleSettings = await responseData(
        await expectStatus(
            await directRequest(reportsBase, "reports", "/api/reports/settings", "reports:schedule:view"),
            200,
            "schedule installation settings",
        ),
        "schedule installation settings",
    );
    if (scheduleSettings.timezone !== "UTC") {
        throw new Error(`worker verifier requires its UTC fixture timezone, got ${scheduleSettings.timezone}`);
    }

    const utcParts = (date) => ({
        dueTime: `${String(date.getUTCHours()).padStart(2, "0")}:${String(date.getUTCMinutes()).padStart(2, "0")}`,
        weekday: (date.getUTCDay() + 6) % 7,
    });
    const backdateScheduleFixture = (id, dueTime) => {
        // This disposable service-verification database makes a post-downtime slot
        // deterministic. HTTP owns every public schedule transition and readback.
        const result = spawnSync([
            "sqlite3",
            reportsDatabase,
            `UPDATE automation_schedules SET due_time='${dueTime}', effective_at='2000-01-01T00:00:00+00:00' WHERE id='${id}';`,
        ]);
        if (result.exitCode !== 0) {
            throw new Error(`could not prepare missed schedule fixture: ${result.stderr.toString()}`);
        }
    };
    const createSchedule = async (cadence, input) => responseData(
        await expectStatus(
            await directRequest(reportsBase, "reports", "/api/reports/schedules", "reports:schedule:manage", {
                method: "POST",
                body: JSON.stringify(input),
            }),
            200,
            `${cadence} schedule creation`,
        ),
        `${cadence} schedule creation`,
    );
    const scheduleList = await responseData(
        await expectStatus(
            await directRequest(reportsBase, "reports", "/api/reports/schedules", "reports:schedule:view"),
            200,
            "schedule list",
        ),
        "schedule list",
    );
    if (!Array.isArray(scheduleList)) throw new Error("schedule list was not an array");

    const nextMinute = new Date(now().getTime() + 60_000);
    nextMinute.setUTCSeconds(0, 0);
    const dailyInput = { flowId: flow.id, cadence: "daily", weekday: null, dueTime: utcParts(nextMinute).dueTime, input: {}, enabled: false };
    const daily = await createSchedule("daily", dailyInput);
    if (daily.enabled || daily.nextDue !== null || daily.cadence !== "daily") {
        throw new Error("daily: disabled schedule state mismatch");
    }
    const dailyPath = `/api/reports/schedules/${daily.id}`;
    const readDaily = await responseData(
        await expectStatus(
            await directRequest(reportsBase, "reports", dailyPath, "reports:schedule:view"),
            200,
            "daily schedule read",
        ),
        "daily schedule read",
    );
    if (readDaily.id !== daily.id) throw new Error("daily schedule readback mismatched its created id");
    await expectStatus(await directRequest(reportsBase, "reports", dailyPath, "reports:schedule:view", {
        method: "PUT", body: JSON.stringify(dailyInput),
    }), 403, "schedule read capability cannot mutate");
    const enabledDaily = await responseData(await expectStatus(await directRequest(
        reportsBase, "reports", dailyPath, "reports:schedule:manage",
        { method: "PUT", body: JSON.stringify({ ...dailyInput, enabled: true }) },
    ), 200, "enable daily schedule"), "enabled daily schedule");
    if (!enabledDaily.enabled || !enabledDaily.nextDue || !enabledDaily.timezone) {
        throw new Error("daily: enabled schedule lacks next occurrence/timezone");
    }
    const enqueuedDaily = await waitForScheduleDecision({ directRequest, expectStatus, responseData, reportsBase, id: daily.id, decision: "enqueued", sleep });
    if (!enqueuedDaily.lastOccurrence?.runId || enqueuedDaily.lastRun?.id !== enqueuedDaily.lastOccurrence.runId) {
        throw new Error(`daily enqueued occurrence lost its run linkage: ${JSON.stringify(enqueuedDaily)}`);
    }
    const sourceRunId = enqueuedDaily.lastOccurrence.runId;
    const sourceSnapshot = spawnSync([
        "sqlite3",
        reportsDatabase,
        `UPDATE automation_runs SET status='failed',error='fixture failure',finished_at='2000-01-01T00:00:00+00:00' WHERE id='${sourceRunId}'; SELECT flow_id || '|' || input_json FROM automation_runs WHERE id='${sourceRunId}';`,
    ]);
    if (sourceSnapshot.exitCode !== 0) throw new Error(`could not prepare retry fixture: ${sourceSnapshot.stderr.toString()}`);
    const retrySourceSnapshot = sourceSnapshot.stdout.toString().trim();
    const retriedRun = await responseData(
        await expectStatus(
            await directRequest(reportsBase, "reports", `/api/reports/runs/${sourceRunId}/retry`, "reports:run:manage", {
                method: "POST",
            }),
            200,
            "retry failed report run",
        ),
        "retry failed report run",
    );
    if (retriedRun.id === sourceRunId || retriedRun.status !== "queued") {
        throw new Error(`retry did not return an independent queued run: ${JSON.stringify(retriedRun)}`);
    }
    const retryCheck = spawnSync([
        "sqlite3",
        reportsDatabase,
        `SELECT flow_id || '|' || input_json FROM automation_runs WHERE id='${retriedRun.id}'; SELECT run_id FROM automation_schedule_occurrences WHERE schedule_id='${daily.id}'; SELECT COUNT(*) FROM automation_schedule_occurrences WHERE run_id='${retriedRun.id}';`,
    ]);
    if (retryCheck.exitCode !== 0) throw new Error(`could not inspect retry fixture: ${retryCheck.stderr.toString()}`);
    const [retriedSnapshot, linkedSourceRunId, retryOccurrences] = retryCheck.stdout.toString().trim().split("\n");
    if (retriedSnapshot !== retrySourceSnapshot || linkedSourceRunId !== sourceRunId || retryOccurrences !== "0") {
        throw new Error("retry changed source snapshot or schedule occurrence linkage");
    }
    const repeatedRetry = await responseData(
        await expectStatus(
            await directRequest(reportsBase, "reports", `/api/reports/runs/${sourceRunId}/retry`, "reports:run:manage", {
                method: "POST",
            }),
            200,
            "repeat retry returns the existing child",
        ),
        "repeat retry returns the existing child",
    );
    if (repeatedRetry.id !== retriedRun.id) {
        throw new Error(`repeat retry created a different child: ${JSON.stringify(repeatedRetry)}`);
    }
    const terminalRetryFixture = spawnSync([
        "sqlite3",
        reportsDatabase,
        `UPDATE automation_runs SET status='failed',error='retry fixture failure',finished_at='2000-01-01T00:00:00+00:00' WHERE id='${retriedRun.id}';`,
    ]);
    if (terminalRetryFixture.exitCode !== 0) throw new Error(`could not finish retry fixture: ${terminalRetryFixture.stderr.toString()}`);
    const terminalRepeatedRetry = await responseData(
        await expectStatus(
            await directRequest(reportsBase, "reports", `/api/reports/runs/${sourceRunId}/retry`, "reports:run:manage", {
                method: "POST",
            }),
            200,
            "terminal child remains the source retry result",
        ),
        "terminal child remains the source retry result",
    );
    if (terminalRepeatedRetry.id !== retriedRun.id || terminalRepeatedRetry.status !== "failed") {
        throw new Error(`terminal retry child was replaced: ${JSON.stringify(terminalRepeatedRetry)}`);
    }
    const chainedRetry = await responseData(
        await expectStatus(
            await directRequest(reportsBase, "reports", `/api/reports/runs/${retriedRun.id}/retry`, "reports:run:manage", {
                method: "POST",
            }),
            200,
            "retry terminal child",
        ),
        "retry terminal child",
    );
    if (chainedRetry.id === retriedRun.id) throw new Error("terminal retry child did not create a new chain link");
    await expectStatus(
        await directRequest(reportsBase, "reports", `/api/reports/runs/${chainedRetry.id}/retry`, "reports:run:manage", {
            method: "POST",
        }),
        409,
        "retry non-terminal report run",
    );

    const missedMoment = new Date(now().getTime() - 120_000);
    const weeklyParts = utcParts(missedMoment);
    const weeklyInput = { flowId: flow.id, cadence: "weekly", weekday: weeklyParts.weekday, dueTime: weeklyParts.dueTime, input: {}, enabled: true };
    const weekly = await createSchedule("weekly", weeklyInput);
    backdateScheduleFixture(weekly.id, weeklyParts.dueTime);
    const skippedWeekly = await waitForScheduleDecision({ directRequest, expectStatus, responseData, reportsBase, id: weekly.id, decision: "skipped", sleep });
    if (skippedWeekly.lastOccurrence?.reason !== "missed" || skippedWeekly.lastOccurrence.runId || skippedWeekly.lastRun) {
        throw new Error(`weekly missed occurrence had an invalid run relationship: ${JSON.stringify(skippedWeekly)}`);
    }
    const disabledWeekly = await responseData(await expectStatus(await directRequest(
        reportsBase, "reports", `/api/reports/schedules/${weekly.id}`, "reports:schedule:manage",
        { method: "PUT", body: JSON.stringify({ ...weeklyInput, enabled: false }) },
    ), 200, "disable weekly schedule"), "disabled weekly schedule");
    if (disabledWeekly.enabled || disabledWeekly.nextDue !== null) throw new Error("weekly schedule did not disable");

    for (const id of [daily.id, weekly.id]) {
        const path = `/api/reports/schedules/${id}`;
        await expectStatus(await directRequest(reportsBase, "reports", path, "reports:schedule:manage", {
            method: "DELETE",
        }), 200, "remove verification schedule");
        await expectStatus(await directRequest(reportsBase, "reports", path, "reports:schedule:view"), 404, "removed schedule");
    }
    console.log("Reports daily/weekly schedule CRUD, idempotent retry chains, permissions, occurrence decisions and run linkage verified");
}
