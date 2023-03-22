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
        if !circular_deps.iter().any(|i| i == node_name) {
            circular_deps.push(node_name.clone());
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

fn merge_circular_paths(
    path: &mut IndexSet<String>,
    circular_path: String,
) -> String {
    let mut path_vec = path.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

    let collect = circular_path
        .split(" > ")
        .map(|s| s.replace('|', ""))
        .collect::<Vec<String>>();

    path_vec.extend(collect.iter().map(|s| s.as_str()).collect::<Vec<&str>>());

    let mut new_path: Vec<String> = vec![];

    for item in path_vec {
        if !new_path.contains(&item.to_string()) {
            new_path.push(item.to_string());
        } else {
            new_path.push(format!("|{}|", item));

            new_path = new_path
                .iter()
                .map(|s| {
                    if s == item {
                        format!("|{}|", s)
                    } else {
                        s.to_string()
                    }
                })
                .collect::<Vec<String>>();
            break;
        }
    }

    new_path.join(" > ")
}

#[cfg(test)]
mod tests;
