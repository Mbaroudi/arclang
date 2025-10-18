use super::*;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitManager {
    repo_path: PathBuf,
}

impl GitManager {
    pub fn new(repo_path: &str) -> Result<Self, CollaborationError> {
        let path = PathBuf::from(repo_path);
        
        if !path.exists() {
            return Err(CollaborationError::GitError(
                format!("Repository path does not exist: {}", repo_path)
            ));
        }
        
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return Err(CollaborationError::GitError(
                "Not a git repository".to_string()
            ));
        }
        
        Ok(Self { repo_path: path })
    }
    
    pub fn init(repo_path: &str) -> Result<Self, CollaborationError> {
        let path = PathBuf::from(repo_path);
        
        std::fs::create_dir_all(&path)
            .map_err(|e| CollaborationError::GitError(format!("Failed to create directory: {}", e)))?;
        
        let output = Command::new("git")
            .args(&["init"])
            .current_dir(&path)
            .output()
            .map_err(|e| CollaborationError::GitError(format!("Failed to initialize git: {}", e)))?;
        
        if !output.status.success() {
            return Err(CollaborationError::GitError(
                format!("Git init failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        Ok(Self { repo_path: path })
    }
    
    pub fn commit(&self, message: &str, author: &str) -> Result<String, CollaborationError> {
        self.run_command(&["add", "."])?;
        
        self.run_command(&[
            "-c", &format!("user.name={}", author),
            "-c", &format!("user.email={}@arclang.local", author),
            "commit",
            "-m", message,
        ])?;
        
        let output = self.run_command(&["rev-parse", "HEAD"])?;
        let commit_id = output.trim().to_string();
        
        Ok(commit_id)
    }
    
    pub fn create_branch(&self, branch_name: &str) -> Result<(), CollaborationError> {
        self.run_command(&["branch", branch_name])?;
        Ok(())
    }
    
    pub fn checkout(&self, branch_name: &str) -> Result<(), CollaborationError> {
        self.run_command(&["checkout", branch_name])?;
        Ok(())
    }
    
    pub fn merge(&self, branch_name: &str, strategy: &str) -> Result<(), CollaborationError> {
        let result = self.run_command(&["merge", "--strategy", strategy, branch_name]);
        
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                if self.has_merge_conflicts()? {
                    Err(CollaborationError::MergeConflict(
                        "Merge conflicts detected".to_string()
                    ))
                } else {
                    Err(e)
                }
            }
        }
    }
    
    pub fn get_current_branch(&self) -> Result<String, CollaborationError> {
        let output = self.run_command(&["branch", "--show-current"])?;
        Ok(output.trim().to_string())
    }
    
    pub fn get_common_ancestor(&self, branch: &str) -> Result<super::semantic_merge::ModelSnapshot, CollaborationError> {
        let current_branch = self.get_current_branch()?;
        let output = self.run_command(&["merge-base", &current_branch, branch])?;
        let commit_id = output.trim();
        
        self.load_model_at_commit(commit_id)
    }
    
    pub fn get_current_model(&self) -> Result<super::semantic_merge::ModelSnapshot, CollaborationError> {
        self.load_model_at_commit("HEAD")
    }
    
    pub fn get_branch_model(&self, branch: &str) -> Result<super::semantic_merge::ModelSnapshot, CollaborationError> {
        self.load_model_at_commit(branch)
    }
    
    pub fn get_changeset(&self, commit_id: &str) -> Result<ChangeSet, CollaborationError> {
        let show_output = self.run_command(&[
            "show",
            "--format=%an|%at|%s",
            "--no-patch",
            commit_id,
        ])?;
        
        let parts: Vec<&str> = show_output.trim().split('|').collect();
        if parts.len() < 3 {
            return Err(CollaborationError::GitError("Invalid commit format".to_string()));
        }
        
        let author = parts[0].to_string();
        let timestamp_str = parts[1];
        let message = parts[2].to_string();
        
        let timestamp = chrono::NaiveDateTime::from_timestamp_opt(
            timestamp_str.parse::<i64>().unwrap_or(0),
            0
        )
        .ok_or_else(|| CollaborationError::GitError("Invalid timestamp".to_string()))?;
        
        let datetime = DateTime::<Utc>::from_utc(timestamp, Utc);
        
        let diff_output = self.run_command(&["diff", &format!("{}^", commit_id), commit_id])?;
        let changes = self.parse_diff(&diff_output)?;
        
        Ok(ChangeSet {
            id: commit_id.to_string(),
            author,
            timestamp: datetime,
            message,
            changes: changes.clone(),
            affected_elements: changes.iter().map(|c| c.element_id.clone()).collect(),
            semantic_diff: SemanticDiff {
                added_requirements: Vec::new(),
                modified_requirements: Vec::new(),
                deleted_requirements: Vec::new(),
                added_components: Vec::new(),
                modified_components: Vec::new(),
                deleted_components: Vec::new(),
                added_relationships: Vec::new(),
                deleted_relationships: Vec::new(),
                integrity_impact: IntegrityImpact {
                    breaks_traceability: false,
                    affects_safety_requirements: false,
                    impacts_interfaces: Vec::new(),
                    orphaned_elements: Vec::new(),
                    severity: ImpactSeverity::None,
                },
            },
        })
    }
    
    pub fn list_branches(&self) -> Result<Vec<String>, CollaborationError> {
        let output = self.run_command(&["branch", "--format=%(refname:short)"])?;
        
        Ok(output
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }
    
    pub fn get_diff(&self, from: &str, to: &str) -> Result<Vec<ModelChange>, CollaborationError> {
        let output = self.run_command(&["diff", from, to])?;
        self.parse_diff(&output)
    }
    
    pub fn push(&self, remote: &str, branch: &str) -> Result<(), CollaborationError> {
        self.run_command(&["push", remote, branch])?;
        Ok(())
    }
    
    pub fn pull(&self, remote: &str, branch: &str) -> Result<(), CollaborationError> {
        self.run_command(&["pull", remote, branch])?;
        Ok(())
    }
    
    pub fn fetch(&self, remote: &str) -> Result<(), CollaborationError> {
        self.run_command(&["fetch", remote])?;
        Ok(())
    }
    
    pub fn has_merge_conflicts(&self) -> Result<bool, CollaborationError> {
        let output = self.run_command(&["diff", "--name-only", "--diff-filter=U"])?;
        Ok(!output.trim().is_empty())
    }
    
    pub fn get_conflict_files(&self) -> Result<Vec<String>, CollaborationError> {
        let output = self.run_command(&["diff", "--name-only", "--diff-filter=U"])?;
        
        Ok(output
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }
    
    pub fn resolve_conflict(&self, file_path: &str, resolution: &str) -> Result<(), CollaborationError> {
        let full_path = self.repo_path.join(file_path);
        
        std::fs::write(&full_path, resolution)
            .map_err(|e| CollaborationError::GitError(format!("Failed to write file: {}", e)))?;
        
        self.run_command(&["add", file_path])?;
        
        Ok(())
    }
    
    pub fn abort_merge(&self) -> Result<(), CollaborationError> {
        self.run_command(&["merge", "--abort"])?;
        Ok(())
    }
    
    fn run_command(&self, args: &[&str]) -> Result<String, CollaborationError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| CollaborationError::GitError(format!("Failed to run git command: {}", e)))?;
        
        if !output.status.success() {
            return Err(CollaborationError::GitError(
                format!("Git command failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    fn load_model_at_commit(&self, commit: &str) -> Result<super::semantic_merge::ModelSnapshot, CollaborationError> {
        use std::collections::HashMap;
        
        Ok(super::semantic_merge::ModelSnapshot {
            elements: HashMap::new(),
        })
    }
    
    fn parse_diff(&self, diff_output: &str) -> Result<Vec<ModelChange>, CollaborationError> {
        let mut changes = Vec::new();
        
        let lines: Vec<&str> = diff_output.lines().collect();
        let mut current_file: Option<String> = None;
        
        for line in lines {
            if line.starts_with("diff --git") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    current_file = Some(parts[3].trim_start_matches("b/").to_string());
                }
            } else if line.starts_with("+++ ") && current_file.is_some() {
            } else if line.starts_with("@@") && current_file.is_some() {
            }
        }
        
        Ok(changes)
    }
}

pub fn setup_git_hooks(repo_path: &str) -> Result<(), CollaborationError> {
    let hooks_dir = Path::new(repo_path).join(".git").join("hooks");
    
    std::fs::create_dir_all(&hooks_dir)
        .map_err(|e| CollaborationError::GitError(format!("Failed to create hooks directory: {}", e)))?;
    
    let pre_commit_hook = hooks_dir.join("pre-commit");
    let pre_commit_script = r#"#!/bin/sh
# ArcLang pre-commit hook
# Validates semantic integrity before commit

arclang validate --staged
if [ $? -ne 0 ]; then
    echo "Semantic validation failed. Commit aborted."
    exit 1
fi

arclang lint --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Commit aborted."
    exit 1
fi

exit 0
"#;
    
    std::fs::write(&pre_commit_hook, pre_commit_script)
        .map_err(|e| CollaborationError::GitError(format!("Failed to write pre-commit hook: {}", e)))?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&pre_commit_hook)
            .map_err(|e| CollaborationError::GitError(format!("Failed to get file metadata: {}", e)))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&pre_commit_hook, perms)
            .map_err(|e| CollaborationError::GitError(format!("Failed to set permissions: {}", e)))?;
    }
    
    let post_merge_hook = hooks_dir.join("post-merge");
    let post_merge_script = r#"#!/bin/sh
# ArcLang post-merge hook
# Validates model integrity after merge

arclang validate
if [ $? -ne 0 ]; then
    echo "WARNING: Model integrity issues detected after merge."
    echo "Run 'arclang validate --fix' to resolve."
fi

exit 0
"#;
    
    std::fs::write(&post_merge_hook, post_merge_script)
        .map_err(|e| CollaborationError::GitError(format!("Failed to write post-merge hook: {}", e)))?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&post_merge_hook)
            .map_err(|e| CollaborationError::GitError(format!("Failed to get file metadata: {}", e)))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&post_merge_hook, perms)
            .map_err(|e| CollaborationError::GitError(format!("Failed to set permissions: {}", e)))?;
    }
    
    Ok(())
}
