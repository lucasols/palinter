use std::collections::HashMap;

use indexmap::IndexSet;

#[derive(Debug, PartialEq)]
pub struct DepsResult {
    pub deps: IndexSet<String>,
    pub circular_deps: Option<Vec<String>>,
}

pub fn get_node_deps<F>(
    start: &String,
    get_node_edges: &mut F,
    deps_cache: &mut HashMap<String, IndexSet<String>>,
) -> Result<DepsResult, String>
where
    F: FnMut(&str) -> Result<Vec<String>, String>,
{
    let mut visited = IndexSet::new();
    let mut circular_deps: Vec<String> = Vec::new();
    let mut path = IndexSet::new();

    // if let Some(deps) = deps_cache.get(start) {
    //     return Ok(DepsResult {
    //         deps: deps.clone(),
    //         circular_deps: None,
    //     });
    // }

    dfs(
        &mut visited,
        start,
        0,
        get_node_edges,
        &mut circular_deps,
        &mut path,
        deps_cache,
    )?;

    Ok(DepsResult {
        deps: visited,
        circular_deps: (!circular_deps.is_empty()).then_some(circular_deps),
    })
}

fn dfs<F>(
    visited: &mut IndexSet<String>,
    node_name: &String,
    depth: usize,
    get_node_edges: &mut F,
    circular_deps: &mut Vec<String>,
    path: &mut IndexSet<String>,
    deps_cache: &mut HashMap<String, IndexSet<String>>,
) -> Result<Option<IndexSet<String>>, String>
where
    F: FnMut(&str) -> Result<Vec<String>, String>,
{
    if visited.contains(node_name) {
        return Ok(None);
    }

    if path.contains(node_name) {
        let mut circular_path: Vec<String> = path.clone().iter().cloned().collect();

        circular_path.push(node_name.to_string());

        visited.insert(node_name.to_string());

        circular_deps.push(
            circular_path
                .iter()
                .map(|s| {
                    if s == node_name {
                        format!("|{}|", s)
                    } else {
                        s.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join(" > "),
        );
        return Ok(None);
    }

    path.insert(node_name.to_string());

    if depth != 0 {
        visited.insert(node_name.to_string());
    }

    let edges = get_node_edges(node_name)?;

    if edges.is_empty() {
        return Ok(Some(IndexSet::new()));
    }

    // let mut has_circular_deps = true;
    // let mut deps = IndexSet::new();

    for edge in edges {
        if let Some(edge_deps) = dfs(
            visited,
            &edge,
            depth + 1,
            get_node_edges,
            circular_deps,
            path,
            deps_cache,
        )? {
            // deps_cache.insert(edge.to_string(), edge_deps.clone());
            // deps.extend(edge_deps);
            // has_circular_deps = false;
        }

        // deps.insert(edge.to_string());
    }

    // if !has_circular_deps {
    //     return Ok(Some(deps));
    // }

    Ok(None)
}

#[cfg(test)]
mod tests;
