use anyhow::{Context, Result};
use ciron_common::{GlobalConfig, ProgramConfig};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub enum ProcessEvent {
    Started(String),
    Exited(String, i32),
    Failed(String, String),
    RestartRequested(String),
}

pub struct ProcessManager {
    processes: HashMap<String, ManagedProcess>,
    event_tx: mpsc::UnboundedSender<ProcessEvent>,
    event_rx: mpsc::UnboundedReceiver<ProcessEvent>,
}

struct ManagedProcess {
    _name: String,
    config: ProgramConfig,
    monitor_handle: Option<JoinHandle<()>>,
    pid: Option<u32>,
    running: bool,
}

impl ProcessManager {
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Self {
            processes: HashMap::new(),
            event_tx,
            event_rx,
        }
    }

    pub fn load_from_config(&mut self, config: GlobalConfig) {
        for (name, program_config) in config.program {
            info!("Loaded program configuration: {}", name);
            self.processes.insert(
                name.clone(),
                ManagedProcess {
                    _name: name,
                    config: program_config,
                    monitor_handle: None,
                    pid: None,
                    running: false,
                },
            );
        }
    }

    pub async fn start_all_autostart(&mut self) -> Result<()> {
        let programs_to_start: Vec<String> = self
            .processes
            .iter()
            .filter(|(_, p)| p.config.autostart)
            .map(|(name, _)| name.clone())
            .collect();

        for name in programs_to_start {
            if let Err(e) = self.start_process(&name).await {
                error!("Failed to start autostart program {}: {}", name, e);
            }
        }

        Ok(())
    }

    pub async fn start_process(&mut self, name: &str) -> Result<()> {
        let process = self
            .processes
            .get_mut(name)
            .context(format!("Program {} not found", name))?;

        if process.running {
            warn!("Process {} is already running", name);
            return Ok(());
        }

        info!("Starting process: {}", name);

        // Parse command and arguments using shell-words for proper quote handling
        let parts = shell_words::split(&process.config.command)
            .context(format!("Failed to parse command for {}", name))?;

        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty command for {}", name));
        }

        let mut cmd = Command::new(&parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }

        // Set environment variables if specified
        if let Some(env) = &process.config.env {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        let mut child = cmd
            .spawn()
            .context(format!("Failed to spawn process {}", name))?;

        let pid = child.id();
        info!("Process {} started with PID: {:?}", name, pid);

        // Store the PID
        process.pid = pid;

        let _ = self.event_tx.send(ProcessEvent::Started(name.to_string()));

        // Start monitoring the process
        let name_clone = name.to_string();
        let event_tx = self.event_tx.clone();

        let monitor_handle = tokio::spawn(async move {
            match child.wait().await {
                Ok(status) => {
                    let code = status.code().unwrap_or(-1);
                    info!("Process {} exited with code: {}", name_clone, code);
                    let _ = event_tx.send(ProcessEvent::Exited(name_clone.clone(), code));
                }
                Err(e) => {
                    error!("Error waiting for process {}: {}", name_clone, e);
                    let _ = event_tx.send(ProcessEvent::Failed(name_clone.clone(), e.to_string()));
                }
            }
        });

        process.monitor_handle = Some(monitor_handle);
        process.running = true;

        Ok(())
    }

    pub async fn stop_process(&mut self, name: &str) -> Result<()> {
        let process = self
            .processes
            .get_mut(name)
            .context(format!("Program {} not found", name))?;

        if !process.running {
            warn!("Process {} is not running", name);
            return Ok(());
        }

        info!("Stopping process: {}", name);

        // Send SIGTERM to the process
        if let Some(pid) = process.pid {
            #[cfg(unix)]
            {
                use nix::sys::signal::{Signal, kill};
                use nix::unistd::Pid;

                match kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                    Ok(_) => info!("Sent SIGTERM to process {} (PID: {})", name, pid),
                    Err(e) => warn!(
                        "Failed to send SIGTERM to process {} (PID: {}): {}",
                        name, pid, e
                    ),
                }
            }

            #[cfg(not(unix))]
            {
                warn!("Signal handling not implemented for non-Unix systems");
            }
        }

        // Cancel the monitor task
        if let Some(handle) = process.monitor_handle.take() {
            handle.abort();
        }

        process.running = false;
        process.pid = None;

        Ok(())
    }

    pub async fn restart_process(&mut self, name: &str) -> Result<()> {
        info!("Restarting process: {}", name);
        self.stop_process(name).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        self.start_process(name).await?;
        Ok(())
    }

    pub async fn handle_single_event(&mut self) -> bool {
        match self.event_rx.try_recv() {
            Ok(event) => {
                match event {
                    ProcessEvent::Started(name) => {
                        info!("Event: Process {} started", name);
                    }
                    ProcessEvent::Exited(name, code) => {
                        info!("Event: Process {} exited with code {}", name, code);

                        // Mark process as not running
                        if let Some(process) = self.processes.get_mut(&name) {
                            process.running = false;

                            // Handle restart policy
                            let should_restart = match process.config.restart.as_deref() {
                                Some("always") => true,
                                Some("on-failure") => code != 0,
                                _ => false,
                            };

                            if should_restart {
                                info!(
                                    "Scheduling restart for process {} due to restart policy",
                                    name
                                );
                                let event_tx = self.event_tx.clone();
                                let name_clone = name.clone();
                                tokio::spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                    let _ =
                                        event_tx.send(ProcessEvent::RestartRequested(name_clone));
                                });
                            }
                        }
                    }
                    ProcessEvent::Failed(name, error) => {
                        error!("Event: Process {} failed: {}", name, error);
                        if let Some(process) = self.processes.get_mut(&name) {
                            process.running = false;
                        }
                    }
                    ProcessEvent::RestartRequested(name) => {
                        info!("Event: Restart requested for {}", name);
                        if let Err(e) = self.start_process(&name).await {
                            error!("Failed to restart process {}: {}", name, e);
                        }
                    }
                }
                true
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => false,
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                info!("Event channel closed");
                false
            }
        }
    }

    pub fn list_processes(&self) -> Vec<(String, bool)> {
        self.processes
            .iter()
            .map(|(name, process)| (name.clone(), process.running))
            .collect()
    }

    pub async fn stop_all(&mut self) {
        info!("Stopping all processes");
        let names: Vec<String> = self.processes.keys().cloned().collect();
        for name in names {
            if let Err(e) = self.stop_process(&name).await {
                error!("Failed to stop process {}: {}", name, e);
            }
        }
    }

    pub fn shutdown(&mut self) {
        info!("Shutting down process manager");
        self.event_rx.close();
    }
}
