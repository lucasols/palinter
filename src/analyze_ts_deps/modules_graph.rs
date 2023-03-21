use indexmap::IndexSet;

#[derive(Debug, PartialEq)]
pub struct DepsResult {
    pub deps: IndexSet<String>,
    pub circular_deps: Option<Vec<String>>,
}

pub fn get_node_deps<F>(
    start: &String,
    get_node_edges: &F,
) -> Result<DepsResult, String>
where
    F: Fn(&str) -> Result<Vec<String>, String>,
{
    let mut visited = IndexSet::new();
    let mut circular_deps: Vec<String> = Vec::new();
    let mut path = IndexSet::new();

    dfs(
        &mut visited,
        start,
        0,
        get_node_edges,
        &mut circular_deps,
        &mut path,
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
    get_node_edges: &F,
    circular_deps: &mut Vec<String>,
    path: &mut IndexSet<String>,
) -> Result<(), String>
where
    F: Fn(&str) -> Result<Vec<String>, String>,
{
    if visited.contains(node_name) {
        return Ok(());
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
        return Ok(());
    }

    path.insert(node_name.to_string());

    if depth != 0 {
        visited.insert(node_name.to_string());
    }

    for edge in &get_node_edges(node_name)? {
        dfs(
            visited,
            edge,
            depth + 1,
            get_node_edges,
            circular_deps,
            path,
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use indexmap::IndexMap;
    use insta::assert_debug_snapshot;
    use pretty_assertions::assert_eq;

    fn sfv(v: Vec<&str>) -> IndexSet<String> {
        vc(v).into_iter().collect()
    }

    fn vc(v: Vec<&str>) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    const EMPTY: Vec<String> = vec![];

    fn get_deps_for_each(
        nodes: Vec<&str>,
        get_node_edges: impl Fn(&str) -> Result<Vec<String>, String>,
    ) -> IndexMap<String, DepsResult> {
        nodes
            .iter()
            .map(|node| {
                (
                    node.to_string(),
                    get_node_deps(&node.to_string(), &get_node_edges).unwrap(),
                )
            })
            .collect()
    }

    #[test]
    fn simple_dep_calc_1() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "dep1" => vc(["dep2"].to_vec()),
                "dep2" => vc(["dep3"].to_vec()),
                "dep3" => EMPTY,
                _ => EMPTY,
            })
        };

        assert_eq!(
            get_node_deps(&"dep1".to_string(), &get_node_edges).unwrap(),
            DepsResult {
                deps: sfv(["dep2", "dep3"].to_vec()),
                circular_deps: None
            }
        );
        assert_eq!(
            get_node_deps(&"dep2".to_string(), &get_node_edges).unwrap(),
            DepsResult {
                deps: sfv(["dep3"].to_vec()),
                circular_deps: None
            }
        );
        assert_eq!(
            get_node_deps(&"dep3".to_string(), &get_node_edges).unwrap(),
            DepsResult {
                deps: sfv(vec![]),
                circular_deps: None
            }
        );
    }

    #[test]
    fn simple_dep_calc_2() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "dep1" => vc(["dep2", "dep4"].to_vec()),
                "dep2" => vc(["dep3"].to_vec()),
                "dep3" => EMPTY,
                "dep4" => vc(["dep5"].to_vec()),
                "dep5" => EMPTY,
                _ => EMPTY,
            })
        };

        assert_eq!(
            get_node_deps(&"dep1".to_string(), &get_node_edges).unwrap(),
            DepsResult {
                deps: sfv(["dep2", "dep3", "dep4", "dep5"].to_vec()),
                circular_deps: None
            }
        );
        assert_eq!(
            get_node_deps(&"dep2".to_string(), &get_node_edges).unwrap(),
            DepsResult {
                deps: sfv(["dep3"].to_vec()),
                circular_deps: None
            }
        );
        assert_eq!(
            get_node_deps(&"dep3".to_string(), &get_node_edges).unwrap(),
            DepsResult {
                deps: sfv(vec![]),
                circular_deps: None
            }
        );
        assert_eq!(
            get_node_deps(&"dep4".to_string(), &get_node_edges).unwrap(),
            DepsResult {
                deps: sfv(["dep5"].to_vec()),
                circular_deps: None
            }
        );
        assert_eq!(
            get_node_deps(&"dep5".to_string(), &get_node_edges).unwrap(),
            DepsResult {
                deps: sfv(vec![]),
                circular_deps: None
            }
        );
    }

    #[test]
    fn circular_dep_calc() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "circular" => vc(["dep1"].to_vec()),
                "dep1" => vc(["dep2"].to_vec()),
                "dep2" => vc(["circular"].to_vec()),
                _ => EMPTY,
            })
        };

        assert_debug_snapshot!(get_node_deps(
            &"circular".to_string(),
            &get_node_edges
        ).unwrap(), @r###"
        DepsResult {
            deps: {
                "dep1",
                "dep2",
                "circular",
            },
            circular_deps: Some(
                [
                    "|circular| > dep1 > dep2 > |circular|",
                ],
            ),
        }
        "###);

        assert_debug_snapshot!(get_node_deps(&"dep1".to_string(), &get_node_edges).unwrap(), @r###"
        DepsResult {
            deps: {
                "dep2",
                "circular",
                "dep1",
            },
            circular_deps: Some(
                [
                    "|dep1| > dep2 > circular > |dep1|",
                ],
            ),
        }
        "###);

        assert_debug_snapshot!(get_node_deps(&"dep2".to_string(), &get_node_edges).unwrap(), @r###"
        DepsResult {
            deps: {
                "circular",
                "dep1",
                "dep2",
            },
            circular_deps: Some(
                [
                    "|dep2| > circular > dep1 > |dep2|",
                ],
            ),
        }
        "###);
    }

    #[test]
    fn circular_1() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "circular" => vc(["dep1"].to_vec()),
                "dep1" => vc(["dep2", "dep3"].to_vec()),
                "dep2" => vc(["circular"].to_vec()),
                "dep3" => EMPTY,
                _ => EMPTY,
            })
        };

        assert_debug_snapshot!(get_node_deps(
            &"circular".to_string(),
            &get_node_edges
        ).unwrap(), @r###"
        DepsResult {
            deps: {
                "dep1",
                "dep2",
                "circular",
                "dep3",
            },
            circular_deps: Some(
                [
                    "|circular| > dep1 > dep2 > |circular|",
                ],
            ),
        }
        "###);

        assert_debug_snapshot!(get_node_deps(&"dep1".to_string(), &get_node_edges).unwrap(), @r###"
        DepsResult {
            deps: {
                "dep2",
                "circular",
                "dep1",
                "dep3",
            },
            circular_deps: Some(
                [
                    "|dep1| > dep2 > circular > |dep1|",
                ],
            ),
        }
        "###);

        assert_debug_snapshot!(get_node_deps(&"dep2".to_string(), &get_node_edges).unwrap(), @r###"
        DepsResult {
            deps: {
                "circular",
                "dep1",
                "dep2",
                "dep3",
            },
            circular_deps: Some(
                [
                    "|dep2| > circular > dep1 > |dep2|",
                ],
            ),
        }
        "###);

        assert_debug_snapshot!(get_node_deps(&"dep3".to_string(), &get_node_edges).unwrap(), @r###"
        DepsResult {
            deps: {},
            circular_deps: None,
        }
        "###);
    }

    #[test]
    fn circular_2() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "dep1" => vc(["dep2"].to_vec()),
                "dep2" => vc(["dep3"].to_vec()),
                "dep3" => vc(["dep2"].to_vec()),
                _ => EMPTY,
            })
        };

        assert_debug_snapshot!(
            get_node_deps(&"dep1".to_string(), &get_node_edges).unwrap(),
            @r###"
        DepsResult {
            deps: {
                "dep2",
                "dep3",
            },
            circular_deps: None,
        }
        "###
        );

        assert_debug_snapshot!(
            get_node_deps(&"dep2".to_string(), &get_node_edges).unwrap(),
            @r###"
            DepsResult {
                deps: {
                    "dep3",
                    "dep2",
                },
                circular_deps: Some(
                    [
                        "|dep2| > dep3 > |dep2|",
                    ],
                ),
            }
            "###
        );

        assert_debug_snapshot!(
            get_node_deps(&"dep3".to_string(), &get_node_edges).unwrap(),
            @r###"
            DepsResult {
                deps: {
                    "dep2",
                    "dep3",
                },
                circular_deps: Some(
                    [
                        "|dep3| > dep2 > |dep3|",
                    ],
                ),
            }
            "###
        );
    }

    #[test]
    fn circular_3() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "dep1" => vc(["dep2"].to_vec()),
                "dep2" => vc(["dep3", "dep4"].to_vec()),
                "dep3" => vc(["dep2"].to_vec()),
                "dep4" => vc(["dep2", "dep5"].to_vec()),
                "dep5" => vc(["dep1"].to_vec()),
                _ => EMPTY,
            })
        };

        assert_debug_snapshot!(
            get_node_deps(&"dep1".to_string(), &get_node_edges).unwrap(),
            @r###"
        DepsResult {
            deps: {
                "dep2",
                "dep3",
                "dep4",
                "dep5",
                "dep1",
            },
            circular_deps: Some(
                [
                    "|dep1| > dep2 > dep3 > dep4 > dep5 > |dep1|",
                ],
            ),
        }
        "###);

        assert_debug_snapshot!(get_node_deps(&"dep2".to_string(), &get_node_edges).unwrap(), @r###"
            DepsResult {
                deps: {
                    "dep3",
                    "dep2",
                    "dep4",
                    "dep5",
                    "dep1",
                },
                circular_deps: Some(
                    [
                        "|dep2| > dep3 > |dep2|",
                        "|dep2| > dep4 > |dep2|",
                        "|dep2| > dep4 > dep5 > dep1 > |dep2|",
                    ],
                ),
            }
            "###);

        assert_debug_snapshot!(
            get_node_deps(&"dep3".to_string(), &get_node_edges).unwrap(),
            @r###"
            DepsResult {
                deps: {
                    "dep2",
                    "dep3",
                    "dep4",
                    "dep5",
                    "dep1",
                },
                circular_deps: Some(
                    [
                        "|dep3| > dep2 > |dep3|",
                        "dep3 > |dep2| > dep4 > |dep2|",
                        "dep3 > |dep2| > dep4 > dep5 > dep1 > |dep2|",
                    ],
                ),
            }
            "###
        );

        assert_debug_snapshot!(
        get_node_deps(&"dep4".to_string(), &get_node_edges).unwrap(),
            @r###"
            DepsResult {
                deps: {
                    "dep2",
                    "dep3",
                    "dep4",
                    "dep5",
                    "dep1",
                },
                circular_deps: Some(
                    [
                        "dep4 > |dep2| > dep3 > |dep2|",
                        "|dep4| > dep2 > |dep4|",
                        "dep4 > dep5 > dep1 > |dep2| > dep3 > |dep2|",
                        "|dep4| > dep5 > dep1 > dep2 > |dep4|",
                    ],
                ),
            }
            "###
        );

        assert_debug_snapshot!(
            get_node_deps(&"dep5".to_string(), &get_node_edges).unwrap(),
            @r###"
            DepsResult {
                deps: {
                    "dep1",
                    "dep2",
                    "dep3",
                    "dep4",
                    "dep5",
                },
                circular_deps: Some(
                    [
                        "dep5 > dep1 > |dep2| > dep3 > |dep2|",
                        "dep5 > dep1 > |dep2| > dep4 > |dep2|",
                        "|dep5| > dep1 > dep2 > dep4 > |dep5|",
                    ],
                ),
            }
            "###
        );
    }

    #[test]
    fn circular_4() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "dep1" => vc(["dep2"].to_vec()),
                "dep2" => vc(["dep3", "dep4"].to_vec()),
                "dep3" => vc(["dep2"].to_vec()),
                "dep4" => vc(["dep2", "dep5"].to_vec()),
                "dep5" => vc(["dep3"].to_vec()),
                _ => EMPTY,
            })
        };

        assert_debug_snapshot!(
        get_node_deps(&"dep1".to_string(), &get_node_edges).unwrap(),
        @r###"
        DepsResult {
            deps: {
                "dep2",
                "dep3",
                "dep4",
                "dep5",
            },
            circular_deps: None,
        }
        "###
        );

        assert_debug_snapshot!(
        get_node_deps(&"dep2".to_string(), &get_node_edges).unwrap(),
        @r###"
            DepsResult {
                deps: {
                    "dep3",
                    "dep2",
                    "dep4",
                    "dep5",
                },
                circular_deps: Some(
                    [
                        "|dep2| > dep3 > |dep2|",
                        "|dep2| > dep4 > |dep2|",
                        "|dep2| > dep4 > dep5 > dep3 > |dep2|",
                    ],
                ),
            }
            "###
        );

        assert_debug_snapshot!(
        get_node_deps(&"dep3".to_string(), &get_node_edges).unwrap(),
        @r###"
            DepsResult {
                deps: {
                    "dep2",
                    "dep3",
                    "dep4",
                    "dep5",
                },
                circular_deps: Some(
                    [
                        "|dep3| > dep2 > |dep3|",
                        "dep3 > |dep2| > dep4 > |dep2|",
                        "|dep3| > dep2 > dep4 > dep5 > |dep3|",
                    ],
                ),
            }
            "###
        );

        assert_debug_snapshot!(
        get_node_deps(&"dep4".to_string(), &get_node_edges).unwrap(),
        @r###"
            DepsResult {
                deps: {
                    "dep2",
                    "dep3",
                    "dep4",
                    "dep5",
                },
                circular_deps: Some(
                    [
                        "dep4 > |dep2| > dep3 > |dep2|",
                        "|dep4| > dep2 > |dep4|",
                        "dep4 > dep5 > |dep3| > dep2 > |dep3|",
                        "|dep4| > dep5 > dep3 > dep2 > |dep4|",
                    ],
                ),
            }
            "###
        );

        assert_debug_snapshot!(
        get_node_deps(&"dep5".to_string(), &get_node_edges).unwrap(),
        @r###"
            DepsResult {
                deps: {
                    "dep3",
                    "dep2",
                    "dep4",
                    "dep5",
                },
                circular_deps: Some(
                    [
                        "dep5 > |dep3| > dep2 > |dep3|",
                        "dep5 > dep3 > |dep2| > dep4 > |dep2|",
                        "|dep5| > dep3 > dep2 > dep4 > |dep5|",
                    ],
                ),
            }
            "###
        );
    }

    #[test]
    fn non_cyclic_graph() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "a" => vc(["b", "c"].to_vec()),
                "b" => vc(["d"].to_vec()),
                "c" => vc(["d"].to_vec()),
                "d" => EMPTY,
                _ => EMPTY,
            })
        };

        assert_debug_snapshot!(
            get_node_deps(&"a".to_string(), &get_node_edges).unwrap(),
            @r###"
                DepsResult {
                    deps: {
                        "b",
                        "d",
                        "c",
                    },
                    circular_deps: None,
                }
                "###
        );

        assert_debug_snapshot!(
            get_node_deps(&"b".to_string(), &get_node_edges).unwrap(),
            @r###"
                DepsResult {
                    deps: {
                        "d",
                    },
                    circular_deps: None,
                }
                "###
        );

        assert_debug_snapshot!(
        get_node_deps(&"c".to_string(), &get_node_edges).unwrap(),
        @r###"
                DepsResult {
                    deps: {
                        "d",
                    },
                    circular_deps: None,
                }
                "###
            );

        assert_debug_snapshot!(
        get_node_deps(&"d".to_string(), &get_node_edges).unwrap(),
        @r###"
                DepsResult {
                    deps: {},
                    circular_deps: None,
                }
                "###
            );
    }

    #[test]
    fn dag_5() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "dropdown" => vc(["popover", "typings", "uOCO", "uDVU"].to_vec()),
                "popover" => vc(["portalLayer"].to_vec()),
                "portalLayer" => EMPTY,
                "typings" => EMPTY,
                "uOCO" => EMPTY,
                "uDVU" => vc(["useTimout"].to_vec()),
                "useTimout" => vc(["typings"].to_vec()),
                _ => EMPTY,
            })
        };

        assert_debug_snapshot!(
            get_deps_for_each(
                ["dropdown", "popover", "portalLayer", "typings", "uOCO", "uDVU", "useTimout"].to_vec(),
                get_node_edges
            ),
            @r###"
        {
            "dropdown": DepsResult {
                deps: {
                    "popover",
                    "portalLayer",
                    "typings",
                    "uOCO",
                    "uDVU",
                    "useTimout",
                },
                circular_deps: None,
            },
            "popover": DepsResult {
                deps: {
                    "portalLayer",
                },
                circular_deps: None,
            },
            "portalLayer": DepsResult {
                deps: {},
                circular_deps: None,
            },
            "typings": DepsResult {
                deps: {},
                circular_deps: None,
            },
            "uOCO": DepsResult {
                deps: {},
                circular_deps: None,
            },
            "uDVU": DepsResult {
                deps: {
                    "useTimout",
                    "typings",
                },
                circular_deps: None,
            },
            "useTimout": DepsResult {
                deps: {
                    "typings",
                },
                circular_deps: None,
            },
        }
        "###
        );
    }

    #[test]
    fn simple_6() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "dep1" => vc(["dep2", "dep4"].to_vec()),
                "dep2" => vc(["dep3"].to_vec()),
                "dep3" => EMPTY,
                "dep4" => vc(["dep2"].to_vec()),
                _ => EMPTY,
            })
        };

        assert_debug_snapshot!(
            get_deps_for_each(
                ["dep1", "dep2", "dep3", "dep4"].to_vec(),
                get_node_edges
            ),
            @r###"
        {
            "dep1": DepsResult {
                deps: {
                    "dep2",
                    "dep3",
                    "dep4",
                },
                circular_deps: None,
            },
            "dep2": DepsResult {
                deps: {
                    "dep3",
                },
                circular_deps: None,
            },
            "dep3": DepsResult {
                deps: {},
                circular_deps: None,
            },
            "dep4": DepsResult {
                deps: {
                    "dep2",
                    "dep3",
                },
                circular_deps: None,
            },
        }
        "###
        );
    }

    #[test]
    fn simple_7() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "dep1" => vc(["dep2", "dep4"].to_vec()),
                "dep2" => vc(["dep3"].to_vec()),
                "dep3" => EMPTY,
                "dep4" => vc(["dep5"].to_vec()),
                "dep5" => EMPTY,
                _ => EMPTY,
            })
        };

        assert_debug_snapshot!(
            get_deps_for_each(
                ["dep1", "dep2", "dep3", "dep4", "dep5"].to_vec(),
                get_node_edges
            ),
            @r###"
            {
                "dep1": DepsResult {
                    deps: {
                        "dep2",
                        "dep3",
                        "dep4",
                        "dep5",
                    },
                    circular_deps: None,
                },
                "dep2": DepsResult {
                    deps: {
                        "dep3",
                    },
                    circular_deps: None,
                },
                "dep3": DepsResult {
                    deps: {},
                    circular_deps: None,
                },
                "dep4": DepsResult {
                    deps: {
                        "dep5",
                    },
                    circular_deps: None,
                },
                "dep5": DepsResult {
                    deps: {},
                    circular_deps: None,
                },
            }
            "###);
    }

    #[test]
    fn circular_8() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "circular" => vc(["dep1"].to_vec()),
                "dep1" => vc(["dep2", "dep3"].to_vec()),
                "dep2" => vc(["circular"].to_vec()),
                "dep3" => vc(["dep4"].to_vec()),
                "dep4" => EMPTY,
                _ => EMPTY,
            })
        };

        assert_debug_snapshot!(
            get_deps_for_each(
                ["circular", "dep1", "dep2", "dep3", "dep4"].to_vec(),
                get_node_edges
            ),
            @r###"
            {
                "circular": DepsResult {
                    deps: {
                        "dep1",
                        "dep2",
                        "circular",
                        "dep3",
                        "dep4",
                    },
                    circular_deps: Some(
                        [
                            "|circular| > dep1 > dep2 > |circular|",
                        ],
                    ),
                },
                "dep1": DepsResult {
                    deps: {
                        "dep2",
                        "circular",
                        "dep1",
                        "dep3",
                        "dep4",
                    },
                    circular_deps: Some(
                        [
                            "|dep1| > dep2 > circular > |dep1|",
                        ],
                    ),
                },
                "dep2": DepsResult {
                    deps: {
                        "circular",
                        "dep1",
                        "dep2",
                        "dep3",
                        "dep4",
                    },
                    circular_deps: Some(
                        [
                            "|dep2| > circular > dep1 > |dep2|",
                        ],
                    ),
                },
                "dep3": DepsResult {
                    deps: {
                        "dep4",
                    },
                    circular_deps: None,
                },
                "dep4": DepsResult {
                    deps: {},
                    circular_deps: None,
                },
            }
            "###);
    }

    #[test]
    fn circular_9() {
        let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
            Ok(match node_name {
                "A" => vc(["B", "C"].to_vec()),
                "B" => vc(["D", "E"].to_vec()),
                "C" => vc(["D"].to_vec()),
                "D" => vc(["1"].to_vec()),
                "E" => vc(["F"].to_vec()),
                "F" => vc(["B"].to_vec()),
                "1" => vc(["2"].to_vec()),
                "2" => vc(["3"].to_vec()),
                "3" => vc(["1"].to_vec()),
                _ => EMPTY,
            })
        };

        assert_debug_snapshot!(
            get_deps_for_each(
                ["A", "B", "C", "D", "E", "F", "1", "2", "3"].to_vec(),
                get_node_edges
            ),
            @r###"
        {
            "A": DepsResult {
                deps: {
                    "B",
                    "D",
                    "1",
                    "2",
                    "3",
                    "E",
                    "F",
                    "C",
                },
                circular_deps: None,
            },
            "B": DepsResult {
                deps: {
                    "D",
                    "1",
                    "2",
                    "3",
                    "E",
                    "F",
                    "B",
                },
                circular_deps: Some(
                    [
                        "|B| > D > 1 > 2 > 3 > E > F > |B|",
                    ],
                ),
            },
            "C": DepsResult {
                deps: {
                    "D",
                    "1",
                    "2",
                    "3",
                },
                circular_deps: None,
            },
            "D": DepsResult {
                deps: {
                    "1",
                    "2",
                    "3",
                },
                circular_deps: None,
            },
            "E": DepsResult {
                deps: {
                    "F",
                    "B",
                    "D",
                    "1",
                    "2",
                    "3",
                    "E",
                },
                circular_deps: Some(
                    [
                        "|E| > F > B > D > 1 > 2 > 3 > |E|",
                    ],
                ),
            },
            "F": DepsResult {
                deps: {
                    "B",
                    "D",
                    "1",
                    "2",
                    "3",
                    "E",
                    "F",
                },
                circular_deps: Some(
                    [
                        "|F| > B > D > 1 > 2 > 3 > E > |F|",
                    ],
                ),
            },
            "1": DepsResult {
                deps: {
                    "2",
                    "3",
                    "1",
                },
                circular_deps: Some(
                    [
                        "|1| > 2 > 3 > |1|",
                    ],
                ),
            },
            "2": DepsResult {
                deps: {
                    "3",
                    "1",
                    "2",
                },
                circular_deps: Some(
                    [
                        "|2| > 3 > 1 > |2|",
                    ],
                ),
            },
            "3": DepsResult {
                deps: {
                    "1",
                    "2",
                    "3",
                },
                circular_deps: Some(
                    [
                        "|3| > 1 > 2 > |3|",
                    ],
                ),
            },
        }
        "###);
    }
}
