use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::{Mutex, OnceLock, RwLock};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Skill {
    pub name: String,
    pub path: String,
    pub description: String,
    pub content: String,
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum SkillError {
    #[error("Skill not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(String),
}

pub struct SkillLoader {
    dir: String,
    pub snapshot: RwLock<HashMap<String, Skill>>,
    pub lru: Mutex<VecDeque<(String, Skill)>>,
    pub lru_capacity: usize,
}

impl SkillLoader {
    pub fn new(dir: &str) -> Result<Self, SkillError> {
        let skills = load_skills_from_disk(dir)?;
        let mut snapshot_map = HashMap::new();
        for skill in skills {
            snapshot_map.insert(skill.name.clone(), skill);
        }
        Ok(Self {
            dir: dir.to_string(),
            snapshot: RwLock::new(snapshot_map),
            lru: Mutex::new(VecDeque::new()),
            lru_capacity: 10,
        })
    }

    pub fn reload(&self) -> Result<(), SkillError> {
        let skills = load_skills_from_disk(&self.dir)?;
        let mut snapshot_map = HashMap::new();
        for skill in skills {
            snapshot_map.insert(skill.name.clone(), skill);
        }
        let mut snapshot = self.snapshot.write().unwrap();
        *snapshot = snapshot_map;

        let mut lru = self.lru.lock().unwrap();
        lru.clear();

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Skill> {
        // 1. Check LRU Cache
        {
            let mut lru = self.lru.lock().unwrap();
            if let Some(pos) = lru.iter().position(|(k, _)| k == name) {
                let item = lru.remove(pos).unwrap();
                lru.push_front(item.clone());
                return Some(item.1);
            }
        }

        // 2. Fallback to Snapshot
        let snapshot = self.snapshot.read().unwrap();
        if let Some(skill) = snapshot.get(name) {
            let mut lru = self.lru.lock().unwrap();
            // Insert to front
            lru.push_front((name.to_string(), skill.clone()));
            if lru.len() > self.lru_capacity {
                lru.pop_back();
            }
            return Some(skill.clone());
        }

        None
    }
}

static REGISTRY: OnceLock<RwLock<HashMap<String, SkillLoader>>> = OnceLock::new();

fn get_registry() -> &'static RwLock<HashMap<String, SkillLoader>> {
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn load_skills(dir: &str) -> Result<Vec<Skill>, SkillError> {
    let registry = get_registry();

    // Check if we already have a loader for this directory
    {
        let read_guard = registry.read().unwrap();
        if let Some(loader) = read_guard.get(dir) {
            let snapshot = loader.snapshot.read().unwrap();
            return Ok(snapshot.values().cloned().collect());
        }
    }

    // Otherwise, create a new SkillLoader and insert it
    let loader = SkillLoader::new(dir)?;
    let skills = {
        let snapshot = loader.snapshot.read().unwrap();
        snapshot.values().cloned().collect::<Vec<Skill>>()
    };

    let mut write_guard = registry.write().unwrap();
    write_guard.insert(dir.to_string(), loader);

    Ok(skills)
}

fn load_skills_from_disk(dir: &str) -> Result<Vec<Skill>, SkillError> {
    let path = Path::new(dir);
    if !path.exists() {
        return Err(SkillError::Io(format!("Directory does not exist: {}", dir)));
    }
    if !path.is_dir() {
        return Err(SkillError::Io(format!("Path is not a directory: {}", dir)));
    }
    let mut skills = Vec::new();
    walk_dir_recursive(path, &mut skills)?;
    Ok(skills)
}

fn walk_dir_recursive(path: &Path, skills: &mut Vec<Skill>) -> Result<(), SkillError> {
    if path.is_dir() {
        let entries = std::fs::read_dir(path).map_err(|e| {
            SkillError::Io(format!(
                "Failed to read directory {}: {}",
                path.display(),
                e
            ))
        })?;
        for entry in entries {
            let entry = entry
                .map_err(|e| SkillError::Io(format!("Failed to read directory entry: {}", e)))?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                walk_dir_recursive(&entry_path, skills)?;
            } else if entry_path.is_file() {
                if let Some(file_name) = entry_path.file_name() {
                    if file_name == "SKILL.md" {
                        let content = std::fs::read_to_string(&entry_path).map_err(|e| {
                            SkillError::Io(format!(
                                "Failed to read file {}: {}",
                                entry_path.display(),
                                e
                            ))
                        })?;
                        let path_str = entry_path.to_string_lossy().to_string();
                        let skill = parse_skill_file(&content, &path_str)?;
                        skills.push(skill);
                    }
                }
            }
        }
    }
    Ok(())
}

fn parse_skill_file(content_str: &str, path: &str) -> Result<Skill, SkillError> {
    let content_normalized = content_str.replace("\r\n", "\n");

    if !content_normalized.starts_with("---\n") {
        return Err(SkillError::Io(format!(
            "File at {} does not start with frontmatter separator '---\\n'",
            path
        )));
    }

    let after_first_line = &content_normalized[4..];
    if let Some(end_index) = after_first_line.find("\n---\n") {
        let frontmatter_section = &after_first_line[..end_index];
        let body_section = &after_first_line[end_index + 5..];

        let mut name = None;
        let mut description = None;

        for line in frontmatter_section.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(stripped) = line.strip_prefix("name:") {
                name = Some(clean_yaml_value(stripped));
            } else if let Some(stripped) = line.strip_prefix("description:") {
                description = Some(clean_yaml_value(stripped));
            }
        }

        let name = match name {
            Some(n) if !n.is_empty() => n,
            _ => {
                return Err(SkillError::Io(format!(
                    "Missing or empty 'name' in frontmatter at {}",
                    path
                )))
            }
        };

        let description = description.unwrap_or_default();

        Ok(Skill {
            name,
            path: path.to_string(),
            description,
            content: body_section.to_string(),
        })
    } else {
        Err(SkillError::Io(format!(
            "Could not find closing '---\\n' separator in frontmatter at {}",
            path
        )))
    }
}

fn clean_yaml_value(val: &str) -> String {
    let mut s = val.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        s = &s[1..s.len() - 1];
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, remove_dir_all, write};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn get_temp_test_dir() -> PathBuf {
        let mut dir = std::env::temp_dir();
        let id = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("swal_skills_test_{}_{}", id, now));
        dir
    }

    #[test]
    fn test_load_skills_success() {
        let temp_dir = get_temp_test_dir();
        create_dir_all(&temp_dir).unwrap();

        let skill1_dir = temp_dir.join("sub1");
        create_dir_all(&skill1_dir).unwrap();
        let skill1_path = skill1_dir.join("SKILL.md");
        write(
            &skill1_path,
            "---\nname: skill_one\ndescription: First skill\n---\nHello from Skill One!",
        )
        .unwrap();

        let skill2_dir = temp_dir.join("sub2/nested");
        create_dir_all(&skill2_dir).unwrap();
        let skill2_path = skill2_dir.join("SKILL.md");
        write(
            &skill2_path,
            "---\nname: \"skill_two\"\ndescription: 'Second skill'\n---\nHello from Skill Two!",
        )
        .unwrap();

        let other_path = skill1_dir.join("OTHER.md");
        write(&other_path, "Should be ignored").unwrap();

        let skills_res = load_skills(temp_dir.to_str().unwrap());
        assert!(skills_res.is_ok());
        let mut skills = skills_res.unwrap();
        assert_eq!(skills.len(), 2);

        skills.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(skills[0].name, "skill_one");
        assert_eq!(skills[0].description, "First skill");
        assert_eq!(skills[0].content, "Hello from Skill One!");

        assert_eq!(skills[1].name, "skill_two");
        assert_eq!(skills[1].description, "Second skill");
        assert_eq!(skills[1].content, "Hello from Skill Two!");

        remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_load_skills_cache_hit() {
        let temp_dir = get_temp_test_dir();
        create_dir_all(&temp_dir).unwrap();

        let skill_dir = temp_dir.join("sub");
        create_dir_all(&skill_dir).unwrap();
        let skill_path = skill_dir.join("SKILL.md");
        write(
            &skill_path,
            "---\nname: cached_skill\ndescription: A cached skill\n---\nContent here.",
        )
        .unwrap();

        let dir_str = temp_dir.to_str().unwrap();

        // First call loads from disk
        let skills1 = load_skills(dir_str).unwrap();
        assert_eq!(skills1.len(), 1);
        assert_eq!(skills1[0].name, "cached_skill");

        // Now delete the temp directory from disk
        remove_dir_all(&temp_dir).unwrap();

        // Second call should hit the cache and still return the skill
        let skills2 = load_skills(dir_str).unwrap();
        assert_eq!(skills2.len(), 1);
        assert_eq!(skills2[0].name, "cached_skill");
    }

    #[test]
    fn test_skill_loader_lru_behavior() {
        let temp_dir = get_temp_test_dir();
        create_dir_all(&temp_dir).unwrap();

        for i in 1..=3 {
            let skill_dir = temp_dir.join(format!("sub{}", i));
            create_dir_all(&skill_dir).unwrap();
            let skill_path = skill_dir.join("SKILL.md");
            write(
                &skill_path,
                format!(
                    "---\nname: skill_{}\ndescription: desc_{}\n---\ncontent_{}",
                    i, i, i
                ),
            )
            .unwrap();
        }

        let loader = SkillLoader::new(temp_dir.to_str().unwrap()).unwrap();
        // Force lru_capacity to 2 to test eviction
        {
            let mut lru = loader.lru.lock().unwrap();
            lru.clear();
        }
        // Actually, let's create a custom SkillLoader with lru_capacity = 2.
        let loader = SkillLoader {
            dir: loader.dir.clone(),
            snapshot: loader.snapshot,
            lru: loader.lru,
            lru_capacity: 2,
        };

        // At first, LRU is empty
        {
            let lru = loader.lru.lock().unwrap();
            assert_eq!(lru.len(), 0);
        }

        // Get skill_1 -> fallback to snapshot, adds to LRU
        let s1 = loader.get("skill_1").unwrap();
        assert_eq!(s1.name, "skill_1");
        {
            let lru = loader.lru.lock().unwrap();
            assert_eq!(lru.len(), 1);
            assert_eq!(lru[0].0, "skill_1");
        }

        // Get skill_2 -> fallback to snapshot, adds to LRU
        let _s2 = loader.get("skill_2").unwrap();
        {
            let lru = loader.lru.lock().unwrap();
            assert_eq!(lru.len(), 2);
            assert_eq!(lru[0].0, "skill_2");
            assert_eq!(lru[1].0, "skill_1");
        }

        // Get skill_1 again -> hit LRU, moves to front
        let _s1_again = loader.get("skill_1").unwrap();
        {
            let lru = loader.lru.lock().unwrap();
            assert_eq!(lru.len(), 2);
            assert_eq!(lru[0].0, "skill_1");
            assert_eq!(lru[1].0, "skill_2");
        }

        // Get skill_3 -> fallback to snapshot, adds to LRU, evicts skill_2 (least recently used)
        let _s3 = loader.get("skill_3").unwrap();
        {
            let lru = loader.lru.lock().unwrap();
            assert_eq!(lru.len(), 2);
            assert_eq!(lru[0].0, "skill_3");
            assert_eq!(lru[1].0, "skill_1");
        }

        remove_dir_all(&temp_dir).unwrap();
    }
}
