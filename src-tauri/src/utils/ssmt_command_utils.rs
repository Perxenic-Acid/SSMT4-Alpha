use std::path::Path;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::config::path_manager::PathManager;

pub struct SSMTCommandUtils;

impl SSMTCommandUtils {
    /// Run a program under SSMT resources folder and wait for completion.
    pub fn run_resources_program(
        program_file_name: &str,
        arguments: &[String],
    ) -> Result<(), String> {
        let program_path = PathManager::ssmt_resources_folder().join(program_file_name);
        Self::run_program(&program_path, arguments)
    }

    fn run_program(program_path: &Path, arguments: &[String]) -> Result<(), String> {
        if !program_path.exists() {
            return Err(format!(
                "Current execute path didn't exists: {}",
                program_path.to_string_lossy()
            ));
        }

        let mut command = Command::new(program_path);
        command.args(arguments);

        #[cfg(windows)]
        {
            // Prevent flashing console window when running CLI tools like texconv.exe.
            command.creation_flags(0x08000000);
        }

        let output = command.output().map_err(|e| {
            format!(
                "Failed to run program {}: {}",
                program_path.to_string_lossy(),
                e
            )
        })?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Err(format!(
            "Program exited with code {:?}: {}\nstdout: {}\nstderr: {}",
            output.status.code(),
            program_path.to_string_lossy(),
            stdout,
            stderr
        ))
    }
}
