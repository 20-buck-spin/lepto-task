use crate::{
    models::{
        queued_task::QueuedTask,
        task_models::{LogLevel, RunStatus, RunningTaskPublic, Task},
    },
    server_only::{
        error::AppError,
        log_handler::{handle_internal_task_log, handle_task_run_log},
        utils::{cleanup_tasks_db, create_task_run, mark_task_run_completed},
    },
    utils::get_run_logs_channel_name,
    ws::{
        default_channel_names::{QUEUED_TASKS_INFO_CHANNEL, RUNNING_PROCESSES},
        ws_context::{broadcast_to_channel, WsContext},
    },
};
use chrono::{DateTime, Utc};
use command_group::AsyncCommandGroup;
use croner::Cron;
use dashmap::DashMap;
use keyed_priority_queue::KeyedPriorityQueue;
use leptos::logging;
use sqlx::{query, query_as, Pool, Sqlite};
use std::{path::PathBuf, str::FromStr, sync::Arc, time::Duration};
use sysinfo::ProcessesToUpdate;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{Mutex, Notify, RwLock},
};
use tokio_util::sync::CancellationToken;

const PROCESS_INFO_REFRESH_FREQUENCY: u64 = 3;
const FLUSH_STAT_TO_DB_TIMER: u64 = 10;

enum QueueState {
    Execute(QueuedTask),
    SleepUntil(DateTime<Utc>),
    WaitEmpty,
}

#[derive(Clone, Debug)]
struct ProcessIdentInfo {
    pub process_id: Option<u32>,
    pub start_time: u64,
}

#[derive(Debug)]
pub struct RunningTaskInfo {
    pub task: Task,
    pub current_cpu_usage: f32,
    pub max_cpu_usage: f32,
    pub max_ram_usage: u64,
    pub current_ram_usage: u64,
    pub started_at: DateTime<Utc>,
    cancellation_token: CancellationToken,
    process_ident_info: ProcessIdentInfo,
}

type RunningTasksMap = DashMap<i64, Arc<RwLock<RunningTaskInfo>>>;

#[derive(Clone)]
pub struct TaskManager {
    pub running_tasks: Arc<RunningTasksMap>,
    pub queued_tasks: Arc<Mutex<KeyedPriorityQueue<i64, QueuedTask>>>,
    queue_modded_notify: Arc<Notify>,
    system_info: Arc<parking_lot::RwLock<sysinfo::System>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            running_tasks: Arc::new(DashMap::new()),
            queued_tasks: Arc::new(Mutex::new(KeyedPriorityQueue::new())),
            queue_modded_notify: Arc::new(Notify::new()),
            system_info: Arc::new(parking_lot::RwLock::new(sysinfo::System::new())),
        }
    }

    async fn refresh_processes(&self) {
        loop {
            let task_arcs: Vec<_> = self
                .running_tasks
                .iter()
                .map(|entry| entry.value().clone())
                .collect();

            let mut managed_pids = Vec::new();
            for task_arc in task_arcs {
                let read_lock = task_arc.read().await;
                if let Some(pid) = read_lock.process_ident_info.process_id {
                    managed_pids.push(sysinfo::Pid::from_u32(pid));
                }
            }

            if !managed_pids.is_empty() {
                let sys_info_clone = self.system_info.clone();

                tokio::task::spawn_blocking(move || {
                    let mut write_lock = sys_info_clone.write();
                    write_lock.refresh_processes(ProcessesToUpdate::Some(&managed_pids), true);
                })
                .await
                .expect("Blocking task panicked on read processes. This should not happen");
            }

            tokio::time::sleep(Duration::from_secs(PROCESS_INFO_REFRESH_FREQUENCY)).await;
        }
    }

    pub async fn init(&self, db: Pool<Sqlite>, ws_context: WsContext) -> Result<(), AppError> {
        // Mark all old runs as cancelled
        if let Err(err) = cleanup_tasks_db(&db).await {
            logging::error!("Failed to cleanup db tasks. Err {:?}", err);
        };

        // Pull all scheduled tasks from db and throw into tasks queue
        let tasks = query_as!(Task, "SELECT * FROM task WHERE is_active=true")
            .fetch_all(&db)
            .await
            .map_err(|e| {
                AppError::TaskExecutorError(format!("Got db error when fetching tasks. Err {}", e))
            })?;

        for task in tasks.into_iter() {
            let cron = Cron::from_str(&task.cron_expr)
                .expect(&format!("Cron str not correct for task {:?}", task));
            self.upsert_task(task, cron).await?;
        }

        // Init the loop to execute the tasks
        let self_clone = self.clone();
        let db_clone = db.clone();
        let ws_clone = ws_context.clone();
        tokio::task::spawn(async move { self_clone.run_scheduled_tasks(db_clone, ws_clone).await });

        // Init loop to broadcast queued
        let self_clone = self.clone();
        let ws_clone = ws_context.clone();
        tokio::task::spawn(async move { self_clone.publish_scheduled_tasks_info(ws_clone).await });

        // Init the loop to broadcast running
        let self_clone = self.clone();
        tokio::task::spawn(async move { self_clone.publish_running_tasks_info(ws_context).await });

        // Init system processes info update loop
        let self_clone = self.clone();
        tokio::task::spawn(async move { self_clone.refresh_processes().await });

        // Init loop to flush stats to db (for cases when server process dies)
        let self_clone = self.clone();
        let db_clone = db.clone();
        tokio::task::spawn(async move { self_clone.flush_ongoing_run_stats_to_db(db_clone).await });

        Ok(())
    }

    pub async fn view_running_tasks(&self) -> Vec<RunningTaskPublic> {
        let mut running_tasks_public = vec![];
        let tasks_to_check: Vec<(i64, Arc<RwLock<RunningTaskInfo>>)> = self
            .running_tasks
            .iter()
            .map(|el| (el.key().clone(), el.value().clone()))
            .collect();

        for (key, val) in tasks_to_check {
            let task = val.read().await;

            running_tasks_public.push(RunningTaskPublic {
                task: task.task.clone(),
                run_id: key.clone(),
                max_cpu_usage: task.max_cpu_usage,
                max_ram_usage: task.max_ram_usage,
                current_ram_usage: task.current_ram_usage,
                current_cpu_usage: task.current_cpu_usage,
                started_at: task.started_at,
            });
        }
        running_tasks_public
    }

    async fn publish_running_tasks_info(&self, ws_context: WsContext) {
        loop {
            let running_tasks_public = self.view_running_tasks().await;
            match serde_json::to_value(running_tasks_public) {
                Ok(payload) => {
                    broadcast_to_channel(
                        &ws_context,
                        RUNNING_PROCESSES,
                        crate::ws::WsMessage::Data {
                            channel: RUNNING_PROCESSES.into(),
                            payload: payload,
                        },
                    )
                    .await;
                }
                Err(err) => {
                    // failed to serialize
                    logging::error!("Failed to serialize running tasks Err {}", err)
                }
            };
            tokio::time::sleep(Duration::from_secs(PROCESS_INFO_REFRESH_FREQUENCY)).await;
        }
    }

    pub async fn view_scheduled_task_info(&self) -> Vec<QueuedTask> {
        let queued_tasks: Vec<QueuedTask> = {
            self.queued_tasks
                .lock()
                .await
                .iter()
                .map(|el| el.1.clone())
                .collect()
        };
        queued_tasks
    }

    async fn publish_scheduled_tasks_info(&self, ws_context: WsContext) {
        loop {
            let queued_tasks = self.view_scheduled_task_info().await;
            match serde_json::to_value(queued_tasks) {
                Ok(payload) => {
                    broadcast_to_channel(
                        &ws_context,
                        QUEUED_TASKS_INFO_CHANNEL,
                        crate::ws::WsMessage::Data {
                            channel: QUEUED_TASKS_INFO_CHANNEL.into(),
                            payload: payload,
                        },
                    )
                    .await;
                }
                Err(err) => {
                    // failed to serialize
                    logging::error!("Failed to serialize queued tasks Err {}", err)
                }
            }
            tokio::time::sleep(Duration::from_secs(PROCESS_INFO_REFRESH_FREQUENCY)).await;
        }
    }

    async fn run_scheduled_tasks(&self, db: Pool<Sqlite>, ws_context: WsContext) {
        loop {
            let notify_fut = self.queue_modded_notify.notified();
            let action = {
                let mut queue_lock = self.queued_tasks.lock().await;
                if let Some((_, next_task)) = queue_lock.peek() {
                    if next_task.next_exec_dt <= Utc::now() {
                        let (_, task) = queue_lock.pop().unwrap(); // guaranteed to be safe because behind peek on the same readlock
                        QueueState::Execute(task)
                    } else {
                        QueueState::SleepUntil(next_task.next_exec_dt)
                    }
                } else {
                    QueueState::WaitEmpty
                }
            };

            match action {
                QueueState::Execute(queued_task) => {
                    let db_clone = db.clone();
                    let ws_clone = ws_context.clone();
                    let self_clone = self.clone();
                    let task_clone = queued_task.task.clone();

                    tokio::spawn(async move {
                        self_clone.run_task(task_clone, db_clone, ws_clone).await
                    });

                    match Cron::from_str(&queued_task.task.cron_expr) {
                        Ok(cron) => {
                            if let Err(err) = self.upsert_task(queued_task.task.clone(), cron).await
                            {
                                logging::error!("Failed to add scheduled task: {:?}", err);
                            };
                        }
                        Err(err) => {
                            logging::error!("Failed to get cron expr: {}", err)
                        }
                    }
                }
                QueueState::SleepUntil(next_exec_dt) => {
                    let remaining = next_exec_dt.signed_duration_since(Utc::now());
                    let sleep_dur = std::cmp::max(
                        remaining.to_std().unwrap_or_default(),
                        Duration::from_millis(1),
                    );

                    tokio::select! {
                        _ = tokio::time::sleep(sleep_dur) => {},
                        _ = notify_fut => {}
                    };
                }
                QueueState::WaitEmpty => {
                    notify_fut.await;
                }
            }
        }
    }

    pub async fn upsert_task(&self, task: Task, cron: Cron) -> Result<(), AppError> {
        match task.is_active {
            true => {
                let next_exec_dt = cron
                    .find_next_occurrence(&Utc::now(), false)
                    .map_err(|_| AppError::CronExprError)?;
                self.queued_tasks
                    .lock()
                    .await
                    .push(task.id.clone(), QueuedTask { task, next_exec_dt });
                self.queue_modded_notify.notify_waiters();
                return Ok(());
            }
            false => {
                self.queued_tasks.lock().await.remove(&task.id);
                return Ok(());
            }
        }
    }

    pub async fn cancel_ongoing_task(self, run_id: i64) -> Result<(), AppError> {
        // Kill any ongoing tasks based on command from the frontend
        let task_arc = self.running_tasks.get(&run_id).map(|r| r.value().clone());
        if let Some(task) = task_arc {
            let read_lock = task.read().await;
            read_lock.cancellation_token.cancel();
        }
        Ok(())
    }

    pub async fn flush_ongoing_run_stats_to_db(&self, db: Pool<Sqlite>) {
        loop {
            tokio::time::sleep(Duration::from_secs(FLUSH_STAT_TO_DB_TIMER)).await;

            let Ok(mut tx) = db.begin().await.inspect_err(|err| {
                logging::log!("Failed to get db transaction on flush. Err {}", err)
            }) else {
                continue;
            };

            let running_tasks: Vec<(i64, Arc<RwLock<RunningTaskInfo>>)> = {
                self.running_tasks
                    .iter()
                    .map(|el| (el.key().clone(), el.value().clone()))
                    .collect()
            };

            for (run_id, run) in running_tasks {
                let run_read_lock = run.read().await;
                let max_cpu_usage = run_read_lock.max_cpu_usage;
                let max_ram_usage = run_read_lock.max_ram_usage as i64;
                if let Err(err) = query!(
                    "
                        UPDATE task_run
                        SET max_cpu_percent = ?,
                            max_ram_bytes = ?

                        WHERE id = ?
                    ",
                    max_cpu_usage,
                    max_ram_usage,
                    run_id,
                )
                .execute(&mut *tx)
                .await
                {
                    logging::log!(
                        "Err on add flush stat statement for run, skipping flush. Err {}",
                        err
                    );
                    continue;
                };
            }

            if let Err(err) = tx.commit().await {
                logging::log!("Failed to commit flush stats to db. Err {}", err)
            }
        }
    }

    async fn gather_running_task_info(&self, run_id: i64) {
        loop {
            let task_arc = match self.running_tasks.get(&run_id) {
                Some(r) => r.value().clone(),
                None => {
                    // Finished or cancelled
                    break;
                }
            };

            let (p_ident, mut max_ram, mut max_cpu) = {
                let read_lock = task_arc.read().await;
                (
                    read_lock.process_ident_info.clone(),
                    read_lock.max_ram_usage,
                    read_lock.max_cpu_usage,
                )
            };

            if let Some(pid) = p_ident.process_id {
                let self_clone = self.clone();
                let process_data = tokio::task::spawn_blocking(move || {
                    let sys_lock = self_clone.system_info.read();
                    if let Some(process_info) = sys_lock.process(sysinfo::Pid::from_u32(pid)) {
                        if process_info.start_time() == p_ident.start_time {
                            Some((process_info.cpu_usage(), process_info.memory()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .await
                .expect("Unexpected failure");

                if let Some((current_cpu, current_ram)) = process_data {
                    let mut write_lock = task_arc.write().await;
                    max_ram = max_ram.max(current_ram);
                    max_cpu = max_cpu.max(current_cpu);
                    write_lock.current_cpu_usage = current_cpu;
                    write_lock.current_ram_usage = current_ram;
                    write_lock.max_cpu_usage = max_cpu;
                    write_lock.max_ram_usage = max_ram;
                } else {
                    // Process not found in sys info
                    // Safe to loop
                    // Current loop exists after child.await() occurs in run_task
                }
            } else {
                // Process ID not found for this run
                // Or internal start time and OS start time differ
                break;
            }
            tokio::time::sleep(Duration::from_secs(PROCESS_INFO_REFRESH_FREQUENCY)).await;
        }
    }

    async fn get_process_ident_info(&self, pid: u32) -> ProcessIdentInfo {
        let mut start_time = 0;
        let sys_pid = sysinfo::Pid::from_u32(pid);

        for _ in 0..5 {
            let mut local_sys_info = sysinfo::System::new();
            let found_start_time = tokio::task::spawn_blocking(move || {
                local_sys_info.refresh_processes(ProcessesToUpdate::Some(&[sys_pid]), true);
                local_sys_info.process(sys_pid).map(|p| p.start_time())
            })
            .await
            .expect("Blocking task panicked on get_process_ident_info");

            if let Some(time) = found_start_time {
                if time > 0 {
                    start_time = time;
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        if start_time == 0 {
            logging::warn!("Failed to capture start_time for PID {}", pid);
        }

        ProcessIdentInfo {
            process_id: Some(pid),
            start_time,
        }
    }

    pub async fn run_task(
        &self,
        task: Task,
        db: Pool<Sqlite>,
        ws_context: WsContext,
    ) -> Result<(), AppError> {
        // Validation to check if working dir is a valid path
        if !PathBuf::from(&task.working_dir).is_dir() {
            return Err(AppError::TaskExecutorError(format!(
                "Task command dir is not a dir"
            )));
        };

        let task_run_created_dt = Utc::now();
        let run_id = create_task_run(&db, task.id, &task_run_created_dt).await?;
        let task_logs_channel = get_run_logs_channel_name(&run_id);

        let parsed_args = shlex::split(&task.command)
            .ok_or_else(|| AppError::TaskExecutorError("Invalid command format".to_string()))?
            .into_iter();
        let mut task_cmd_strs = parsed_args.into_iter();
        let mut child = match Command::new(task_cmd_strs.next().ok_or(
            AppError::TaskExecutorError(format!("Got empty command on task")),
        )?)
        .args(task_cmd_strs.collect::<Vec<String>>())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(&task.working_dir)
        .kill_on_drop(true)
        .group_spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = mark_task_run_completed(&db, run_id, RunStatus::Failed, 0.0, 0).await;
                return Err(AppError::TaskExecutorError(format!(
                    "Failed to launch task. Err {}",
                    e
                )));
            }
        };

        ws_context.add_channel(&task_logs_channel);
        let inner_child = child.inner();

        let process_ident_info = match inner_child.id() {
            Some(pid) => self.get_process_ident_info(pid).await,
            None => ProcessIdentInfo {
                process_id: None,
                start_time: 0,
            },
        };

        // Add the process info to the dict
        let cancellation_token = CancellationToken::new();
        self.running_tasks.insert(
            run_id.clone(),
            Arc::new(RwLock::new(RunningTaskInfo {
                task,
                current_cpu_usage: 0.0,
                max_cpu_usage: 0.0,
                max_ram_usage: 0,
                current_ram_usage: 0,
                started_at: task_run_created_dt,
                cancellation_token: cancellation_token.clone(),
                process_ident_info,
            })),
        );

        let db_clone = db.clone();
        let ws_clone = ws_context.clone();
        let run_id_clone = run_id.clone();
        if let Some(stdout) = inner_child.stdout.take() {
            let stdout_reader = BufReader::new(stdout);
            tokio::spawn(async move {
                let mut lines = stdout_reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    handle_task_run_log(&db_clone, &ws_clone, &run_id_clone, line).await
                }
            });
        };

        let db_clone = db.clone();
        let ws_clone = ws_context.clone();
        let run_id_clone = run_id.clone();
        if let Some(stderr) = inner_child.stderr.take() {
            let stderr_reader = BufReader::new(stderr);
            tokio::spawn(async move {
                let mut lines = stderr_reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    handle_task_run_log(&db_clone, &ws_clone, &run_id_clone, line).await
                }
            });
        };

        // Gather task runtime info
        let self_clone = self.clone();
        tokio::task::spawn(async move { self_clone.gather_running_task_info(run_id).await });

        let running_tasks_clone = self.running_tasks.clone();
        let run_id_clone = run_id.clone();
        let ws_clone = ws_context.clone();
        let db_clone = db.clone();
        tokio::task::spawn(async move {
            let run_exit_status: RunStatus;
            tokio::select! {
                exit_status = child.wait() => {
                    let status = match exit_status {
                        Ok(status) => {
                            if status.success() {
                                handle_internal_task_log(&run_id, "Run Success".into(), LogLevel::Info, &ws_clone, &db_clone).await;
                                RunStatus::Completed
                            } else {
                                handle_internal_task_log(&run_id, "Run failed".into(), LogLevel::Info, &ws_clone, &db_clone).await;
                                RunStatus::Failed
                            }
                        },
                        Err(_) => {
                            handle_internal_task_log(&run_id, "Run Failed".into(), LogLevel::Info, &ws_clone, &db_clone).await;
                            RunStatus::Failed
                        }
                    };
                    run_exit_status = status;
                    // kill spawned processes in the prcoess group as well
                    let _ = child.kill().await;
                }
                _ = cancellation_token.cancelled() => {
                    // Send log that task was killed externally
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    handle_internal_task_log(&run_id, "Run Terminated".into(), LogLevel::Info, &ws_clone, &db_clone).await;
                    run_exit_status = RunStatus::Terminated;
                }
            }

            let (max_cpu_percent, max_ram_bytes) = match running_tasks_clone.remove(&run_id) {
                Some((_, ongoing_run)) => {
                    let read_lock = ongoing_run.read().await;
                    (
                        read_lock.max_cpu_usage as f64,
                        read_lock.max_ram_usage as i64,
                    )
                }
                None => (0.0, 0),
            };
            if let Err(err) = mark_task_run_completed(
                &db_clone,
                run_id_clone,
                run_exit_status,
                max_cpu_percent,
                max_ram_bytes,
            )
            .await
            {
                logging::error!("{}", err);
            };

            ws_clone.remove_channel(&task_logs_channel);
        });
        Ok(())
    }
}
