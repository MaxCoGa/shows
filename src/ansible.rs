use serde_json::Value;
use tokio::process::Command;

#[derive(Debug)]
pub enum AnsibleError {
    CommandError(std::io::Error),
    ExecutionError(String),
    JsonParseError(serde_json::Error),
}

impl From<std::io::Error> for AnsibleError {
    fn from(err: std::io::Error) -> Self {
        AnsibleError::CommandError(err)
    }
}

impl From<serde_json::Error> for AnsibleError {
    fn from(err: serde_json::Error) -> Self {
        AnsibleError::JsonParseError(err)
    }
}

pub async fn run_deployment(container_name: &str, container_image: &str) -> Result<(), AnsibleError> {
    let extra_vars = format!(
        "container_name={} container_image={}",
        container_name,
        container_image
    );

    let mut command = Command::new("ansible-playbook");
    command
        .arg("-i")
        .arg("ansible/inventory.ini")
        .arg("ansible/playbook.yml")
        .arg("--extra-vars")
        .arg(extra_vars);

    let output = command.output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Log stdout and stderr for debugging.
    println!("Ansible stdout: {}", stdout);
    println!("Ansible stderr: {}", stderr);

    if output.status.success() {
        Ok(())
    } else {
        let error_message = format!(
            "Exit code: {:?}. Stderr: {}. Stdout: {}",
            output.status.code(),
            stderr,
            stdout
        );
        Err(AnsibleError::ExecutionError(error_message))
    }
}

pub async fn list_containers() -> Result<Vec<Value>, AnsibleError> {
    let mut command = Command::new("ansible-playbook");
    command
        .env("ANSIBLE_STDOUT_CALLBACK", "json")
        .arg("-i")
        .arg("ansible/inventory.ini")
        .arg("ansible/list_containers.yml");

    let output = command.output().await?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_message = format!(
            "Exit code: {:?}. Stderr: {}. Stdout: {}",
            output.status.code(),
            stderr,
            stdout
        );
        return Err(AnsibleError::ExecutionError(error_message));
    }

    let json_output: Value = serde_json::from_str(&stdout)?;
    let container_lines = json_output["plays"][0]["tasks"][0]["hosts"]["localhost"]["stdout_lines"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let containers: Result<Vec<Value>, _> = container_lines
        .iter()
        .map(|line| serde_json::from_str(line.as_str().unwrap_or("")))
        .collect();

    Ok(containers?)
}
