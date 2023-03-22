## circular_4

```mermaid
graph LR
    dep1-->dep2
    dep2-->dep3
    dep2-->dep4
    dep3-->dep2
    dep4-->dep2
    dep4-->dep5
    dep5-->dep3
```

## non_cyclic_graph

```mermaid
graph LR
    A-->B
    A-->C
    B-->D
    C-->D
```

## dag_5

```mermaid
graph LR
  D[Dropdown] --> T[Typings]
  D --> P[Popover] --> PL[PortalLayer]
  D --> UDV[uDVU]
  D --> UOCO[uOCO]
  UDV --> useTimout --> T
```

# Simple 6

```mermaid
flowchart LR
  Dep1 --> Dep2 & Dep4
  Dep2 --> Dep3
  Dep4 --> Dep2
```

# Simple 7

```mermaid
flowchart LR
  Dep1 --> Dep2 --> Dep3
  Dep1 --> Dep4 --> Dep5
```

# Circular 8

```mermaid
flowchart LR
  circular --> dep1
  dep1 --> dep2 & dep3
  dep2 --> circular
  dep3 --> dep4
```

# Circular 9

```mermaid
flowchart LR
  A --> B & C
    B --> D
    B --> E
    E --> F
    F --> B
    C --> D
    D --> 1
    1 --> 2
    2 --> 3
    3 --> 1
```

# Very complex circular 10

```mermaid
graph TD
    a --> b & c & p
    b --> c & d
    c --> b & d & e
    d --> b & c & e & f
    e --> c & d & f & g
    f --> d & e & g & h
    g --> e & f & h & i
    h --> f & g & i & j
    i --> g & h & j & k
    j --> h & i & k & l
    k --> i & j & l & m
    l --> j & k & m & n
    m --> k & l & n & o
    n --> l & m & o & p
    o --> m & n & p & b
    p --> n & o & c
```

# Very complex circular 13

```mermaid
graph TD
    a --> b & c
    b --> c & d
    c --> b & d
```

# circular_11

```mermaid
graph LR
    A --> B & C
    B --> D & E
    C --> 1 & 3
    E --> F --> E
    1 --> 2 --> 1
```
