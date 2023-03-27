use crate::{analyze_ts_deps::_setup_test, test_utils::TEST_MUTEX};

use super::*;
use indexmap::IndexMap;
use insta::assert_debug_snapshot;

fn vc(v: Vec<&str>) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

const EMPTY: Vec<String> = vec![];

fn get_deps_for_each(
    nodes: Vec<&str>,
    mut get_node_edges: impl Fn(&str) -> Result<Vec<String>, String>,
) -> IndexMap<String, DepsResult> {
    nodes
        .iter()
        .map(|node| {
            (
                node.to_string(),
                get_node_deps(
                    &node.to_string(),
                    &mut get_node_edges,
                    Some(1000),
                    false,
                    false,
                )
                .unwrap(),
            )
        })
        .collect()
}

#[test]
fn simple_dep_calc_1() {
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

    let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
        Ok(match node_name {
            "dep1" => vc(["dep2"].to_vec()),
            "dep2" => vc(["dep3"].to_vec()),
            "dep3" => EMPTY,
            _ => EMPTY,
        })
    };

    assert_debug_snapshot!(
        get_deps_for_each(
            ["dep1", "dep2", "dep3"].to_vec(),
            get_node_edges,
        ),
        @r###"
    {
        "dep1": DepsResult {
            deps: {
                "dep2",
                "dep3",
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
    }
    "###
    )
}

#[test]
fn simple_dep_calc_2() {
    let _guard = TEST_MUTEX.lock().unwrap();
    _setup_test();

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
            get_node_edges,
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
        "###
    )
}

#[test]
fn circular_dep_calc() {
    let _guard = TEST_MUTEX.lock().unwrap();
    _setup_test();

    let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
        Ok(match node_name {
            "circular" => vc(["dep1"].to_vec()),
            "dep1" => vc(["dep2"].to_vec()),
            "dep2" => vc(["circular"].to_vec()),
            _ => EMPTY,
        })
    };

    assert_debug_snapshot!(
        get_deps_for_each(
            ["circular", "dep1", "dep2"].to_vec(),
            get_node_edges,
        ),
        @r###"
    {
        "circular": DepsResult {
            deps: {
                "dep1",
                "dep2",
                "circular",
            },
            circular_deps: Some(
                [
                    "circular",
                ],
            ),
        },
        "dep1": DepsResult {
            deps: {
                "dep2",
                "circular",
                "dep1",
            },
            circular_deps: Some(
                [
                    "circular",
                ],
            ),
        },
        "dep2": DepsResult {
            deps: {
                "circular",
                "dep1",
                "dep2",
            },
            circular_deps: Some(
                [
                    "circular",
                ],
            ),
        },
    }
    "###
    );
}

#[test]
fn circular_1() {
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

    let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
        Ok(match node_name {
            "circular" => vc(["dep1"].to_vec()),
            "dep1" => vc(["dep2", "dep3"].to_vec()),
            "dep2" => vc(["circular"].to_vec()),
            "dep3" => EMPTY,
            _ => EMPTY,
        })
    };

    assert_debug_snapshot!(
        get_deps_for_each(
            ["circular", "dep1", "dep2", "dep3"].to_vec(),
            get_node_edges,
        ),
        @r###"
    {
        "circular": DepsResult {
            deps: {
                "dep1",
                "dep2",
                "circular",
                "dep3",
            },
            circular_deps: Some(
                [
                    "circular",
                ],
            ),
        },
        "dep1": DepsResult {
            deps: {
                "dep2",
                "circular",
                "dep1",
                "dep3",
            },
            circular_deps: Some(
                [
                    "circular",
                ],
            ),
        },
        "dep2": DepsResult {
            deps: {
                "circular",
                "dep1",
                "dep2",
                "dep3",
            },
            circular_deps: Some(
                [
                    "circular",
                ],
            ),
        },
        "dep3": DepsResult {
            deps: {},
            circular_deps: None,
        },
    }
    "###
    );
}

#[test]
fn circular_2() {
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

    let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
        Ok(match node_name {
            "dep1" => vc(["dep2"].to_vec()),
            "dep2" => vc(["dep3"].to_vec()),
            "dep3" => vc(["dep2"].to_vec()),
            _ => EMPTY,
        })
    };

    assert_debug_snapshot!(
        get_deps_for_each(
            ["dep1", "dep2", "dep3"].to_vec(),
            get_node_edges,
        ),
        @r###"
    {
        "dep1": DepsResult {
            deps: {
                "dep2",
                "dep3",
            },
            circular_deps: Some(
                [
                    "dep2",
                ],
            ),
        },
        "dep2": DepsResult {
            deps: {
                "dep3",
                "dep2",
            },
            circular_deps: Some(
                [
                    "dep2",
                ],
            ),
        },
        "dep3": DepsResult {
            deps: {
                "dep2",
                "dep3",
            },
            circular_deps: Some(
                [
                    "dep2",
                ],
            ),
        },
    }
    "###
    );
}

#[test]
fn circular_3() {
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

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
        get_deps_for_each(
            ["dep1", "dep2", "dep3", "dep4", "dep5"].to_vec(),
            get_node_edges,
        ),
        @r###"
    {
        "dep1": DepsResult {
            deps: {
                "dep2",
                "dep3",
                "dep4",
                "dep5",
                "dep1",
            },
            circular_deps: Some(
                [
                    "dep2",
                    "dep1",
                ],
            ),
        },
        "dep2": DepsResult {
            deps: {
                "dep3",
                "dep2",
                "dep4",
                "dep5",
                "dep1",
            },
            circular_deps: Some(
                [
                    "dep2",
                    "dep1",
                ],
            ),
        },
        "dep3": DepsResult {
            deps: {
                "dep2",
                "dep3",
                "dep4",
                "dep5",
                "dep1",
            },
            circular_deps: Some(
                [
                    "dep2",
                    "dep1",
                ],
            ),
        },
        "dep4": DepsResult {
            deps: {
                "dep2",
                "dep3",
                "dep4",
                "dep5",
                "dep1",
            },
            circular_deps: Some(
                [
                    "dep2",
                    "dep1",
                ],
            ),
        },
        "dep5": DepsResult {
            deps: {
                "dep1",
                "dep2",
                "dep3",
                "dep4",
                "dep5",
            },
            circular_deps: Some(
                [
                    "dep2",
                    "dep1",
                ],
            ),
        },
    }
    "###
    );
}

#[test]
fn circular_4() {
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

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
        get_deps_for_each(
            ["dep1", "dep2", "dep3", "dep4", "dep5"].to_vec(),
            get_node_edges,
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
            circular_deps: Some(
                [
                    "dep2",
                ],
            ),
        },
        "dep2": DepsResult {
            deps: {
                "dep3",
                "dep2",
                "dep4",
                "dep5",
            },
            circular_deps: Some(
                [
                    "dep2",
                ],
            ),
        },
        "dep3": DepsResult {
            deps: {
                "dep2",
                "dep3",
                "dep4",
                "dep5",
            },
            circular_deps: Some(
                [
                    "dep2",
                ],
            ),
        },
        "dep4": DepsResult {
            deps: {
                "dep2",
                "dep3",
                "dep4",
                "dep5",
            },
            circular_deps: Some(
                [
                    "dep2",
                ],
            ),
        },
        "dep5": DepsResult {
            deps: {
                "dep3",
                "dep2",
                "dep4",
                "dep5",
            },
            circular_deps: Some(
                [
                    "dep2",
                ],
            ),
        },
    }
    "###
    );
}

#[test]
fn non_cyclic_graph() {
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

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
        get_deps_for_each(
            ["a", "b", "c", "d"].to_vec(),
            get_node_edges,
        ),
        @r###"
        {
            "a": DepsResult {
                deps: {
                    "b",
                    "d",
                    "c",
                },
                circular_deps: None,
            },
            "b": DepsResult {
                deps: {
                    "d",
                },
                circular_deps: None,
            },
            "c": DepsResult {
                deps: {
                    "d",
                },
                circular_deps: None,
            },
            "d": DepsResult {
                deps: {},
                circular_deps: None,
            },
        }
        "###
    );
}

#[test]
fn dag_5() {
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

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
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

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
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

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
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

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
                    "circular",
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
                    "circular",
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
                    "circular",
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
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

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
            circular_deps: Some(
                [
                    "1",
                    "B",
                ],
            ),
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
                    "1",
                    "B",
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
            circular_deps: Some(
                [
                    "1",
                ],
            ),
        },
        "D": DepsResult {
            deps: {
                "1",
                "2",
                "3",
            },
            circular_deps: Some(
                [
                    "1",
                ],
            ),
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
                    "1",
                    "B",
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
                    "1",
                    "B",
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
                    "1",
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
                    "1",
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
                    "1",
                ],
            ),
        },
    }
    "###);
}

#[test]
fn circular_11() {
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

    let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
        Ok(match node_name {
            "A" => vc(["B", "C"].to_vec()),
            "B" => vc(["D", "E"].to_vec()),
            "C" => vc(["1", "3"].to_vec()),
            "D" => EMPTY,
            "E" => vc(["F"].to_vec()),
            "F" => vc(["E"].to_vec()),
            "3" => EMPTY,
            "1" => vc(["2"].to_vec()),
            "2" => vc(["1"].to_vec()),
            _ => EMPTY,
        })
    };

    assert_debug_snapshot!(get_deps_for_each(
        ["A", "B", "C", "D", "E", "F", "1", "2", "3"].to_vec(),
        get_node_edges
    ), @r###"
    {
        "A": DepsResult {
            deps: {
                "B",
                "D",
                "E",
                "F",
                "C",
                "1",
                "2",
                "3",
            },
            circular_deps: Some(
                [
                    "E",
                    "1",
                ],
            ),
        },
        "B": DepsResult {
            deps: {
                "D",
                "E",
                "F",
            },
            circular_deps: Some(
                [
                    "E",
                ],
            ),
        },
        "C": DepsResult {
            deps: {
                "1",
                "2",
                "3",
            },
            circular_deps: Some(
                [
                    "1",
                ],
            ),
        },
        "D": DepsResult {
            deps: {},
            circular_deps: None,
        },
        "E": DepsResult {
            deps: {
                "F",
                "E",
            },
            circular_deps: Some(
                [
                    "E",
                ],
            ),
        },
        "F": DepsResult {
            deps: {
                "E",
                "F",
            },
            circular_deps: Some(
                [
                    "E",
                ],
            ),
        },
        "1": DepsResult {
            deps: {
                "2",
                "1",
            },
            circular_deps: Some(
                [
                    "1",
                ],
            ),
        },
        "2": DepsResult {
            deps: {
                "1",
                "2",
            },
            circular_deps: Some(
                [
                    "1",
                ],
            ),
        },
        "3": DepsResult {
            deps: {},
            circular_deps: None,
        },
    }
    "###);
}

#[test]
fn circular_12() {
    _setup_test();
    let _guard = TEST_MUTEX.lock().unwrap();

    let get_node_edges = |node_name: &str| -> Result<Vec<String>, String> {
        Ok(match node_name {
            "index" => vc(["a"].to_vec()),
            "a" => vc(["b"].to_vec()),
            "b" => vc(["a", "c"].to_vec()),
            "c" => vc(["d"].to_vec()),
            "d" => vc(["c"].to_vec()),
            _ => EMPTY,
        })
    };

    assert_debug_snapshot!(get_deps_for_each(
        ["index", "a", "b", "c", "d"].to_vec(),
        get_node_edges
    ), @r###"
    {
        "index": DepsResult {
            deps: {
                "a",
                "b",
                "c",
                "d",
            },
            circular_deps: Some(
                [
                    "a",
                    "c",
                ],
            ),
        },
        "a": DepsResult {
            deps: {
                "b",
                "a",
                "c",
                "d",
            },
            circular_deps: Some(
                [
                    "a",
                    "c",
                ],
            ),
        },
        "b": DepsResult {
            deps: {
                "a",
                "b",
                "c",
                "d",
            },
            circular_deps: Some(
                [
                    "a",
                    "c",
                ],
            ),
        },
        "c": DepsResult {
            deps: {
                "d",
                "c",
            },
            circular_deps: Some(
                [
                    "c",
                ],
            ),
        },
        "d": DepsResult {
            deps: {
                "c",
                "d",
            },
            circular_deps: Some(
                [
                    "c",
                ],
            ),
        },
    }
    "###);
}
