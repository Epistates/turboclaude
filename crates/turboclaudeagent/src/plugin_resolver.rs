//! Plugin dependency resolution and conflict detection
//!
//! Provides semantic versioning support and safe plugin composition through:
//! - Version constraint matching
//! - Topological sorting for safe loading order
//! - Conflict detection between plugins
//! - Dependency validation

use std::collections::{HashMap, HashSet, VecDeque};

/// Semantic version (e.g., "1.2.3")
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Patch version number
    pub patch: u32,
}

impl Version {
    /// Parse version from string (e.g., "1.2.3")
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err("Version must be in format major.minor.patch".to_string());
        }

        Ok(Version {
            major: parts[0].parse().map_err(|_| "Invalid major version")?,
            minor: parts[1].parse().map_err(|_| "Invalid minor version")?,
            patch: parts[2].parse().map_err(|_| "Invalid patch version")?,
        })
    }

    /// Check if this version matches a constraint (e.g., "^1.0.0", "1.2.3")
    pub fn matches(&self, constraint: &str) -> bool {
        if constraint.starts_with('^') {
            // Caret: compatible with version (same major)
            match Version::parse(&constraint[1..]) {
                Ok(target) => self.major == target.major && self >= &target,
                Err(_) => false,
            }
        } else if constraint.starts_with('~') {
            // Tilde: patch-level changes (same major.minor)
            match Version::parse(&constraint[1..]) {
                Ok(target) => {
                    self.major == target.major && self.minor == target.minor && self >= &target
                }
                Err(_) => false,
            }
        } else if constraint.starts_with('=') {
            // Exact version
            match Version::parse(&constraint[1..]) {
                Ok(target) => self == &target,
                Err(_) => false,
            }
        } else {
            // No prefix means exact match
            match Version::parse(constraint) {
                Ok(target) => self == &target,
                Err(_) => false,
            }
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Plugin manifest with dependencies and version info
#[derive(Debug, Clone)]
pub struct PluginManifest {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: Version,
    /// Plugin description
    pub description: Option<String>,
    /// Required dependencies: plugin_name -> version_constraint
    pub requires: HashMap<String, String>,
    /// Conflicting plugins (cannot be loaded together)
    pub conflicts: Vec<String>,
    /// Priority for resolving conflicts (higher wins)
    pub priority: u32,
}

/// Resolution result after dependency checking
#[derive(Debug)]
pub struct ResolutionPlan {
    /// Topological order for loading plugins
    pub load_order: Vec<String>,
    /// Detected conflicts between plugins
    pub conflicts: Vec<(String, String)>,
    /// Missing or incompatible dependencies
    pub missing: Vec<(String, String)>,
}

/// Plugin dependency resolver
pub struct DependencyResolver {
    manifests: HashMap<String, PluginManifest>,
}

impl DependencyResolver {
    /// Create a new resolver with plugin manifests
    pub fn new(manifests: HashMap<String, PluginManifest>) -> Self {
        Self { manifests }
    }

    /// Resolve dependencies for requested plugins
    pub fn resolve(&self, requested: &[String]) -> Result<ResolutionPlan, String> {
        let mut all_needed = HashSet::new();
        let mut to_process = VecDeque::from_iter(requested.iter().cloned());
        let mut missing = Vec::new();

        // Collect all transitive dependencies
        while let Some(plugin_name) = to_process.pop_front() {
            if all_needed.insert(plugin_name.clone()) {
                match self.manifests.get(&plugin_name) {
                    Some(manifest) => {
                        for dep in manifest.requires.keys() {
                            to_process.push_back(dep.clone());
                        }
                    }
                    None => {
                        missing.push((plugin_name.clone(), "not found".to_string()));
                    }
                }
            }
        }

        if !missing.is_empty() {
            return Err(format!("Missing plugins: {:?}", missing));
        }

        // Check version constraints
        for plugin_name in &all_needed {
            if let Some(manifest) = self.manifests.get(plugin_name) {
                for (dep_name, constraint) in &manifest.requires {
                    match self.manifests.get(dep_name) {
                        Some(dep_manifest) => {
                            if !dep_manifest.version.matches(constraint) {
                                return Err(format!(
                                    "Version mismatch: {} requires {}@{}, found {}",
                                    plugin_name, dep_name, constraint, dep_manifest.version
                                ));
                            }
                        }
                        None => {
                            return Err(format!(
                                "Missing dependency: {} requires {}",
                                plugin_name, dep_name
                            ));
                        }
                    }
                }
            }
        }

        // Check for conflicts
        let mut conflicts = Vec::new();
        for plugin1 in &all_needed {
            if let Some(manifest) = self.manifests.get(plugin1) {
                for plugin2 in &manifest.conflicts {
                    if all_needed.contains(plugin2) {
                        conflicts.push((plugin1.clone(), plugin2.clone()));
                    }
                }
            }
        }

        if !conflicts.is_empty() {
            return Err(format!("Plugin conflicts detected: {:?}", conflicts));
        }

        // Topological sort
        let load_order = self.topological_sort(&all_needed)?;

        Ok(ResolutionPlan {
            load_order,
            conflicts,
            missing,
        })
    }

    /// Topological sort of plugins by dependencies
    fn topological_sort(&self, plugins: &HashSet<String>) -> Result<Vec<String>, String> {
        let mut in_degree = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for plugin in plugins {
            in_degree.insert(plugin.clone(), 0);
            graph.insert(plugin.clone(), Vec::new());
        }

        // Build dependency graph
        for plugin in plugins {
            if let Some(manifest) = self.manifests.get(plugin) {
                for dep in manifest.requires.keys() {
                    if plugins.contains(dep) {
                        graph.get_mut(dep).unwrap().push(plugin.clone());
                        *in_degree.get_mut(plugin).unwrap() += 1;
                    }
                }
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|&(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(plugin) = queue.pop_front() {
            result.push(plugin.clone());

            for dependent in graph.get(&plugin).unwrap().clone() {
                let degree = in_degree.get_mut(&dependent).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(dependent);
                }
            }
        }

        if result.len() != plugins.len() {
            return Err("Circular dependency detected".to_string());
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_matching_caret() {
        let v = Version::parse("1.5.0").unwrap();
        assert!(v.matches("^1.0.0"));
        assert!(v.matches("^1.5.0"));
        assert!(!v.matches("^2.0.0"));
    }

    #[test]
    fn test_version_matching_tilde() {
        let v = Version::parse("1.5.3").unwrap();
        assert!(v.matches("~1.5.0"));
        assert!(!v.matches("~1.4.0"));
    }

    #[test]
    fn test_simple_dependency_resolution() {
        let mut manifests = HashMap::new();

        manifests.insert(
            "plugin-a".to_string(),
            PluginManifest {
                name: "plugin-a".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                description: None,
                requires: HashMap::new(),
                conflicts: vec![],
                priority: 10,
            },
        );

        manifests.insert(
            "plugin-b".to_string(),
            PluginManifest {
                name: "plugin-b".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                description: None,
                requires: {
                    let mut m = HashMap::new();
                    m.insert("plugin-a".to_string(), "^1.0.0".to_string());
                    m
                },
                conflicts: vec![],
                priority: 10,
            },
        );

        let resolver = DependencyResolver::new(manifests);
        let plan = resolver.resolve(&["plugin-b".to_string()]).unwrap();

        assert_eq!(plan.load_order.len(), 2);
        assert_eq!(plan.load_order[0], "plugin-a");
        assert_eq!(plan.load_order[1], "plugin-b");
    }

    #[test]
    fn test_conflict_detection() {
        let mut manifests = HashMap::new();

        manifests.insert(
            "plugin-x".to_string(),
            PluginManifest {
                name: "plugin-x".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                description: None,
                requires: HashMap::new(),
                conflicts: vec!["plugin-y".to_string()],
                priority: 10,
            },
        );

        manifests.insert(
            "plugin-y".to_string(),
            PluginManifest {
                name: "plugin-y".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                description: None,
                requires: HashMap::new(),
                conflicts: vec![],
                priority: 10,
            },
        );

        let resolver = DependencyResolver::new(manifests);
        let result = resolver.resolve(&["plugin-x".to_string(), "plugin-y".to_string()]);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("conflict"));
    }

    #[test]
    fn test_missing_dependency() {
        let mut manifests = HashMap::new();

        manifests.insert(
            "plugin-a".to_string(),
            PluginManifest {
                name: "plugin-a".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                description: None,
                requires: {
                    let mut m = HashMap::new();
                    m.insert("non-existent".to_string(), "^1.0.0".to_string());
                    m
                },
                conflicts: vec![],
                priority: 10,
            },
        );

        let resolver = DependencyResolver::new(manifests);
        let result = resolver.resolve(&["plugin-a".to_string()]);

        assert!(result.is_err());
    }

    #[test]
    fn test_version_mismatch() {
        let mut manifests = HashMap::new();

        manifests.insert(
            "plugin-a".to_string(),
            PluginManifest {
                name: "plugin-a".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                description: None,
                requires: HashMap::new(),
                conflicts: vec![],
                priority: 10,
            },
        );

        manifests.insert(
            "plugin-b".to_string(),
            PluginManifest {
                name: "plugin-b".to_string(),
                version: Version::parse("1.0.0").unwrap(),
                description: None,
                requires: {
                    let mut m = HashMap::new();
                    m.insert("plugin-a".to_string(), "^2.0.0".to_string());
                    m
                },
                conflicts: vec![],
                priority: 10,
            },
        );

        let resolver = DependencyResolver::new(manifests);
        let result = resolver.resolve(&["plugin-b".to_string()]);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Version mismatch"));
    }
}
