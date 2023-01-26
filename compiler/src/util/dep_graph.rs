use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct DependencyGraph {
    pub roots: HashSet<String>,
    pub edges: HashMap<String, HashSet<String>>
}

impl DependencyGraph {

    pub fn add_dependency(&mut self, f: String, dependency: String) {
        self.edges.entry(f).or_insert_with(HashSet::new).insert(dependency);
    }



    pub fn calculate_used(&self) -> HashSet<String> {
        let mut set: HashSet<String> = HashSet::new();
        for root in &self.roots {
            self.bfs(root.clone(), &mut set);
        }
        set
    }

    fn bfs(&self, node: String, visited: &mut HashSet<String>) {
        if visited.contains(&node) {
            return;
        }

        visited.insert(node.clone());

        if let Some(dependencies) = self.edges.get(&node) {
            for dep in dependencies {
                self.bfs(dep.clone(), visited);
            }
        }
    }
}


