use std::{collections::HashMap, sync::Mutex};

use indexmap::IndexSet;
use lazy_static::lazy_static;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct DepsResult {
    pub deps: IndexSet<String>,
    pub circular_deps: Option<Vec<String>>,
}

pub type DepsCache = HashMap<String, DepsResult>;

lazy_static! {
    pub static ref DEPS_CACHE: Mutex<DepsCache> = Mutex::new(HashMap::new());
}

pub fn get_node_deps<F>(
    start: &String,
    get_node_edges: &mut F,
    max_calls: Option<usize>,
    detailed_circular_deps: bool,
    disable_cache: bool,
) -> Result<DepsResult, String>
where
    F: FnMut(&str) -> Result<Vec<String>, String>,
{
    let mut cache = DEPS_CACHE.lock().unwrap();

    if let Some(cached) = cache.get(start) {
        return Ok(cached.clone());
    }

    let mut deps = IndexSet::new();
    let mut circular_deps: Vec<String> = Vec::new();
    let mut path = IndexSet::new();
    let mut calls = 0;

    dfs(
        &mut deps,
        start,
        0,
        get_node_edges,
        &mut circular_deps,
        &mut path,
        max_calls,
        &mut calls,
        &mut cache,
        detailed_circular_deps,
        disable_cache,
    )?;

    let deps_result = DepsResult {
        deps,
        circular_deps: (!circular_deps.is_empty()).then_some(circular_deps),
    };

    cache.insert(start.to_string(), deps_result.clone());

    Ok(deps_result)
}

#[allow(clippy::too_many_arguments)]

fn dfs<F>(
    main_node_deps: &mut IndexSet<String>,
    node_name: &String,
    depth: usize,
    get_node_edges: &mut F,
    circular_deps: &mut Vec<String>,
    path: &mut IndexSet<String>,
    max_calls: Option<usize>,
    calls: &mut usize,
    cache: &mut DepsCache,
    detailed_circular_deps: bool,
    disable_cache: bool,
) -> Result<Option<IndexSet<String>>, String>
where
    F: FnMut(&str) -> Result<Vec<String>, String>,
{
    if let Some(max_calls) = max_calls {
        if *calls >= max_calls {
            panic!("Max calls reached");
        } else {
            *calls += 1;
        }
    }

    if path.contains(node_name) {
        if !detailed_circular_deps {
            if !circular_deps.iter().any(|i| i == node_name) {
                circular_deps.push(node_name.clone());
            }
        } else {
            let mut circular_path: Vec<String> =
                path.clone().iter().cloned().collect();

            circular_path.push(node_name.to_string());

            let circular_path_string = circular_path
                .iter()
                .map(|s| {
                    if s == node_name {
                        format!("|{}|", s)
                    } else {
                        s.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join(" > ");

            if !circular_deps.iter().any(|i| i == &circular_path_string) {
                circular_deps.push(circular_path_string);
            }
        }

        return Ok(None);
    }

    if main_node_deps.contains(node_name) {
        return Ok(None);
    }

    path.insert(node_name.to_string());

    if depth != 0 {
        main_node_deps.insert(node_name.to_string());
    }

    let edges = get_node_edges(node_name)?;

    if !disable_cache {
        if let Some(cached) = cache.get(node_name) {
            main_node_deps.extend(cached.deps.clone());

            path.remove(node_name);

            return if cached.circular_deps.is_some() {
                for circular_path in cached.circular_deps.clone().unwrap() {
                    if !circular_deps.iter().any(|i| i == &circular_path) {
                        circular_deps.push(circular_path);
                    }
                }

                Ok(None)
            } else {
                Ok(Some(cached.deps.clone()))
            };
        }
    }

    let mut has_circular_deps = false;
    let mut deps = IndexSet::new();

    for edge in edges {
        if let Some(edge_deps) = dfs(
            main_node_deps,
            &edge,
            depth + 1,
            get_node_edges,
            circular_deps,
            path,
            max_calls,
            calls,
            cache,
            detailed_circular_deps,
            disable_cache,
        )? {
            cache.insert(
                edge.to_string(),
                DepsResult {
                    deps: edge_deps.clone(),
                    circular_deps: None,
                },
            );
            deps.extend(edge_deps);
        } else {
            has_circular_deps = true;
        }

        deps.insert(edge.to_string());
    }

    path.remove(node_name);

    if !has_circular_deps {
        return Ok(Some(deps));
    }

    Ok(None)
}

#[cfg(test)]
mod tests;
